//! Sprite billboard del enemigo: siempre de cara a la cámara, escalado por
//! distancia, ocluido correctamente detrás de paredes más cercanas vía
//! z-buffer (ver render::walls::render_scene). `assets/sprites/enemy/normal.png`
//! (dibujo de Cami, 900×1600, fondo transparente) es hoy el único estado
//! entregado — el estado `Enraged` reusa la misma imagen con un tinte rojizo
//! (`Texture::set_color_mod`, misma técnica que las paredes) hasta que llegue
//! un dibujo dedicado para ese estado.
//!
//! Igual que las paredes, cada columna copia una tira de 1px de ancho de la
//! textura escalada por GPU — pero acá además hay que recortar verticalmente
//! sin distorsionar cuando el sprite queda más alto que la pantalla (de cerca
//! sobra por arriba/abajo): el rect de origen se recorta a la MISMA fracción
//! vertical que quedó visible en pantalla, en vez de copiar la imagen entera
//! aplastada al espacio recortado.

use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Canvas, Texture};
use sdl2::video::Window;

use crate::engine::camera::camera_view;
use crate::entities::enemy::{Enemy, EnemyState};
use crate::entities::player::Player;

/// Ancho/alto de `normal.png` (800×1400) — si se reemplaza el arte por uno
/// con otra proporción, actualizar este número para que no quede
/// estirado/achatado.
const SPRITE_ASPECT: f64 = 800.0 / 1400.0;

/// Erica, como persona, no debería medir lo mismo que una pared/techo entero
/// — sin esto queda gigante y "flotando" (la cabeza le llega hasta el techo a
/// cualquier distancia, lo que se ve antinatural aunque los pies estén bien
/// apoyados). Fracción de la altura "de pared completa" que ocupa de verdad;
/// bajar este número la hace más chica/humana, subirlo más grande.
const SPRITE_HEIGHT_SCALE: f64 = 0.8;

const ENRAGED_TINT: (u8, u8, u8) = (255, 90, 90); // tinte rojizo, mismo lenguaje que el resto del juego.

/// Dibuja al enemigo como billboard 2D en espacio de pantalla. `z_buffer` debe
/// venir de haber llamado `render::walls::render_scene` este mismo frame.
#[allow(clippy::too_many_arguments)]
pub fn render_enemy(
    canvas: &mut Canvas<Window>,
    texture: &mut Texture,
    player: &Player,
    enemy: &Enemy,
    state: EnemyState,
    anim_flip: bool,
    z_buffer: &[f64],
    w: i32,
    h: i32,
) {
    let view = camera_view(player);

    let rel_x = enemy.x - player.x;
    let rel_y = enemy.y - player.y;

    // Transforma la posición del enemigo a espacio de cámara (igual método que
    // Lodev's raycasting tutorial para sprites): invierte la matriz
    // [dir | plane] para obtener (transform_x, transform_y) — transform_y es
    // la profundidad (distancia a lo largo de la cámara), transform_x la
    // posición lateral.
    let det = view.plane_x * view.dir_y - view.dir_x * view.plane_y;
    if det.abs() < 1e-9 {
        return;
    }
    let inv_det = 1.0 / det;
    let transform_x = inv_det * (view.dir_y * rel_x - view.dir_x * rel_y);
    let transform_y = inv_det * (-view.plane_y * rel_x + view.plane_x * rel_y);

    // Detrás de la cámara (o encima nuestro) — no dibujar.
    if transform_y <= 0.1 {
        return;
    }

    let sprite_screen_x = (w as f64 / 2.0) * (1.0 + transform_x / transform_y);

    // Altura de una pared completa a esta distancia (misma fórmula que
    // render::walls) — de ahí sale dónde cae el piso en pantalla a esta
    // distancia, NO la altura real de Erica (ver SPRITE_HEIGHT_SCALE).
    let full_wall_h = (h as f64 / transform_y).abs();
    let floor_y_f = h as f64 / 2.0 + full_wall_h / 2.0;

    let sprite_h = full_wall_h * SPRITE_HEIGHT_SCALE;
    let sprite_w = sprite_h * SPRITE_ASPECT;

    // Bordes SIN recortar del sprite — se usan para calcular qué fracción de
    // la textura corresponde a la porción visible (ver comentario del
    // módulo). `draw_start_*`/`draw_end_*` son la versión ya recortada a la
    // pantalla, la que efectivamente se dibuja. El borde de ABAJO queda fijo
    // en `floor_y_f` (el piso a esta distancia) y el sprite crece hacia
    // arriba desde ahí — así los pies quedan apoyados sin importar
    // `SPRITE_HEIGHT_SCALE`, en vez de encogerse parejo desde el centro.
    let sprite_top_f = floor_y_f - sprite_h;
    let sprite_left_f = sprite_screen_x - sprite_w / 2.0;

    let draw_start_y = sprite_top_f.max(0.0) as i32;
    let draw_end_y = floor_y_f.min((h - 1) as f64) as i32;
    let draw_start_x = sprite_left_f.max(0.0) as i32;
    let draw_end_x = (sprite_left_f + sprite_w).min((w - 1) as f64) as i32;

    if draw_start_x >= draw_end_x || draw_start_y >= draw_end_y {
        return;
    }

    let tex_query = texture.query();
    let tex_w = tex_query.width as i32;
    let tex_h = tex_query.height as i32;

    // Parpadeo leve de 2 frames (ver SKILL.md: "para que la animación sea
    // inequívoca... dar a cada estado un ciclo de 2 frames alternados") +
    // niebla por distancia, igual criterio que las paredes.
    let flicker = if anim_flip { 1.0 } else { 0.82 };
    let fog = (1.0 - (transform_y / 14.0).min(0.5)).max(0.5);
    let factor = flicker * fog;
    let (tint_r, tint_g, tint_b) = match state {
        EnemyState::Normal => (255.0, 255.0, 255.0),
        EnemyState::Enraged => (ENRAGED_TINT.0 as f64, ENRAGED_TINT.1 as f64, ENRAGED_TINT.2 as f64),
    };
    let shade = |v: f64| ((v * factor).clamp(0.0, 255.0)) as u8;
    texture.set_color_mod(shade(tint_r), shade(tint_g), shade(tint_b));
    canvas.set_blend_mode(BlendMode::Blend); // respeta el canal alfa del PNG.

    // Franja vertical visible, como fracción 0.0-1.0 del sprite completo —
    // recorta la textura de origen a la misma fracción en vez de aplastar la
    // imagen entera al espacio ya recortado (lo que la distorsionaría).
    let v_top = ((draw_start_y as f64 - sprite_top_f) / sprite_h).clamp(0.0, 1.0);
    let v_bottom = ((draw_end_y as f64 - sprite_top_f) / sprite_h).clamp(0.0, 1.0);
    let src_y = (v_top * tex_h as f64) as i32;
    let src_h = (((v_bottom - v_top) * tex_h as f64).max(1.0)) as u32;
    let dest_h = (draw_end_y - draw_start_y).max(1) as u32;

    for x in draw_start_x..draw_end_x {
        if x < 0 || x >= w {
            continue;
        }
        // Ocluido por una pared más cercana en esta columna.
        if transform_y >= z_buffer[x as usize] {
            continue;
        }

        let u = ((x as f64 - sprite_left_f) / sprite_w).clamp(0.0, 1.0);
        let src_x = ((u * tex_w as f64) as i32).clamp(0, tex_w - 1);
        let src = Rect::new(src_x, src_y, 1, src_h);
        let dest = Rect::new(x, draw_start_y, 1, dest_h);
        let _ = canvas.copy(texture, Some(src), Some(dest));
    }
}
