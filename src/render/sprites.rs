//! Sprite billboard del enemigo: siempre de cara a la cámara, escalado por
//! distancia, ocluido correctamente detrás de paredes más cercanas vía
//! z-buffer (ver render::walls::render_scene). Placeholder mientras llega el
//! arte de Cami (Fase 4): silueta procedural cabeza+cuerpo en vez de un
//! rectángulo plano, para que ya se lea como una figura.

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::engine::camera::camera_view;
use crate::entities::enemy::{Enemy, EnemyState};
use crate::entities::player::Player;

const NORMAL_COLOR: (u8, u8, u8) = (95, 35, 35);
const ENRAGED_COLOR: (u8, u8, u8) = (215, 25, 25);

// Proporciones de la silueta placeholder (ver `column_span`).
const SPRITE_ASPECT: f64 = 0.55; // ancho / alto
const HEAD_HEIGHT_FRACTION: f64 = 0.3;
const HEAD_WIDTH_FRACTION: f64 = 0.42;
const SHOULDER_TRIM: f64 = 0.08;

/// Dibuja al enemigo como billboard 2D en espacio de pantalla. `z_buffer` debe
/// venir de haber llamado `render::walls::render_scene` este mismo frame.
pub fn render_enemy(
    canvas: &mut Canvas<Window>,
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

    let sprite_h = (h as f64 / transform_y).abs();
    let sprite_w = sprite_h * SPRITE_ASPECT;

    let draw_start_y = (-sprite_h / 2.0 + h as f64 / 2.0).max(0.0) as i32;
    let draw_end_y = (sprite_h / 2.0 + h as f64 / 2.0).min((h - 1) as f64) as i32;
    let draw_start_x = (sprite_screen_x - sprite_w / 2.0).max(0.0) as i32;
    let draw_end_x = (sprite_screen_x + sprite_w / 2.0).min((w - 1) as f64) as i32;

    if draw_start_x >= draw_end_x || draw_start_y >= draw_end_y {
        return;
    }

    let base = match state {
        EnemyState::Normal => NORMAL_COLOR,
        EnemyState::Enraged => ENRAGED_COLOR,
    };
    // Parpadeo leve de 2 frames (ver SKILL.md: "para que la animación sea
    // inequívoca... dar a cada estado un ciclo de 2 frames alternados").
    let flicker = if anim_flip { 1.0 } else { 0.82 };
    let fog = (1.0 - (transform_y / 14.0).min(0.5)).max(0.5);
    let factor = flicker * fog;
    let color =
        Color::RGB((base.0 as f64 * factor) as u8, (base.1 as f64 * factor) as u8, (base.2 as f64 * factor) as u8);
    canvas.set_draw_color(color);

    let head_split_y = draw_start_y + ((draw_end_y - draw_start_y) as f64 * HEAD_HEIGHT_FRACTION) as i32;
    let sprite_span = (draw_end_x - draw_start_x).max(1) as f64;

    for x in draw_start_x..draw_end_x {
        if x < 0 || x >= w {
            continue;
        }
        // Ocluido por una pared más cercana en esta columna.
        if transform_y >= z_buffer[x as usize] {
            continue;
        }

        let u = (x - draw_start_x) as f64 / sprite_span; // 0..1 a través del sprite
        let centered = (u - 0.5).abs();

        // Torso/piernas: columna central con los hombros levemente recortados.
        if centered <= 0.5 - SHOULDER_TRIM {
            let _ = canvas.fill_rect(Rect::new(x, head_split_y, 1, (draw_end_y - head_split_y).max(1) as u32));
        }
        // Cabeza: solo la franja central angosta, arriba del todo.
        if centered <= HEAD_WIDTH_FRACTION / 2.0 {
            let _ = canvas.fill_rect(Rect::new(x, draw_start_y, 1, (head_split_y - draw_start_y).max(1) as u32));
        }
    }
}
