//! Dibuja la escena 3D: techo (degradado), piso (floor-casting con perspectiva
//! real, para que se vea como un plano que retrocede y no un rectángulo plano) y
//! paredes columna por columna a partir de los impactos del DDA.
//!
//! Fase 1/2: color plano por zona + sombreado de "panel" (placeholder). Fase 4
//! reemplaza `zone_color` por muestreo real de textura (PNGs de Cami + generada
//! por código para la Cocina) usando `RayHit::wall_x` como coordenada U.

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

use crate::engine::camera::camera_view;
use crate::engine::raycasting::{cast_ray, RayHit, Side};
use crate::entities::player::Player;
use crate::map::Zone;

pub const CEILING_TOP: (u8, u8, u8) = (14, 12, 16);
pub const CEILING_HORIZON: (u8, u8, u8) = (34, 28, 34);

// Piso: dos tonos cercanos (madera/loseta vieja) en vez de blanco/negro de
// tablero de ajedrez — encaja mejor con el flat europeo sombrío.
const FLOOR_A: (u8, u8, u8) = (46, 38, 32);
const FLOOR_B: (u8, u8, u8) = (58, 48, 40);

const BASEBOARD_FACTOR: f64 = 0.5; // franja inferior (zócalo)

/// Color base placeholder por zona temática (ver SKILL.md, sección de zonas).
fn zone_color(zone: Zone) -> (u8, u8, u8) {
    match zone {
        Zone::Recibidor => (150, 130, 100), // wallpaper desgastado (Cami)
        Zone::Dormitorio => (110, 45, 45),  // cortinas pesadas (Cami)
        Zone::Sala => (75, 90, 60),         // wallpaper floral (Cami)
        Zone::Cocina => (95, 95, 85),       // sucia/deteriorada (generada)
    }
}

fn lerp_u8(a: u8, b: u8, t: f64) -> u8 {
    (a as f64 + (b as f64 - a as f64) * t.clamp(0.0, 1.0)) as u8
}

fn darken(color: Color, factor: f64) -> Color {
    Color::RGB((color.r as f64 * factor) as u8, (color.g as f64 * factor) as u8, (color.b as f64 * factor) as u8)
}

/// Sombreado base de pared: E/W (Vertical) a color pleno, N/S (Horizontal) más
/// oscuras (como en Wolfenstein), atenuadas por distancia (niebla leve).
fn shaded_color(zone: Zone, side: Side, dist: f64) -> Color {
    let (r, g, b) = zone_color(zone);
    let side_factor = if side == Side::Horizontal { 0.65 } else { 1.0 };
    let fog_factor = (1.0 - (dist / 14.0).min(0.55)).max(0.45);
    let factor = side_factor * fog_factor;
    Color::RGB(((r as f64) * factor) as u8, ((g as f64) * factor) as u8, ((b as f64) * factor) as u8)
}

/// Dibuja el techo como un degradado vertical simple (spec permite "colores
/// planos o degradado simple por código" — no requiere textura dibujada a mano).
fn render_ceiling(canvas: &mut Canvas<Window>, w: i32, h: i32) {
    let horizon = h / 2;
    for y in 0..horizon {
        let t = y as f64 / horizon.max(1) as f64;
        let color = Color::RGB(
            lerp_u8(CEILING_TOP.0, CEILING_HORIZON.0, t),
            lerp_u8(CEILING_TOP.1, CEILING_HORIZON.1, t),
            lerp_u8(CEILING_TOP.2, CEILING_HORIZON.2, t),
        );
        canvas.set_draw_color(color);
        let _ = canvas.fill_rect(Rect::new(0, y, w as u32, 1));
    }
}

/// Floor-casting por píxel (algoritmo estándar, ver Lodev's raycasting tutorial):
/// para cada fila bajo el horizonte se calcula a qué distancia de mundo
/// corresponde, y se proyecta esa fila completa de coordenadas de mundo para
/// pintar un patrón a cuadros que efectivamente se encoge hacia el horizonte —
/// es lo que le da la sensación de piso real en perspectiva (ver nota de Cami:
/// sin esto el piso se ve como un rectángulo plano y "la ilusión se rompe").
fn render_floor(canvas: &mut Canvas<Window>, texture: &mut Texture, buffer: &mut [u8], player: &Player, w: i32, h: i32) {
    let view = camera_view(player);
    let horizon = h / 2;
    let floor_h = h - horizon;
    if floor_h <= 0 {
        return;
    }

    let ray_dir_x0 = view.dir_x - view.plane_x;
    let ray_dir_y0 = view.dir_y - view.plane_y;
    let ray_dir_x1 = view.dir_x + view.plane_x;
    let ray_dir_y1 = view.dir_y + view.plane_y;

    let pitch = (w as usize) * 3;

    for y in 0..floor_h {
        let p = (y + 1) as f64; // distancia (en filas) al horizonte, evita división por 0
        let pos_z = 0.5 * h as f64;
        let row_dist = pos_z / p;

        let floor_step_x = row_dist * (ray_dir_x1 - ray_dir_x0) / w as f64;
        let floor_step_y = row_dist * (ray_dir_y1 - ray_dir_y0) / w as f64;

        let mut floor_x = player.x + row_dist * ray_dir_x0;
        let mut floor_y = player.y + row_dist * ray_dir_y0;

        let fog = (1.0 - (row_dist / 12.0).min(0.6)).max(0.4);
        let row_offset = (y as usize) * pitch;

        for x in 0..w as usize {
            let cell_x = floor_x.floor() as i64;
            let cell_y = floor_y.floor() as i64;
            let checker = (cell_x.wrapping_add(cell_y)) & 1;
            let base = if checker == 0 { FLOOR_A } else { FLOOR_B };

            let idx = row_offset + x * 3;
            buffer[idx] = (base.0 as f64 * fog) as u8;
            buffer[idx + 1] = (base.1 as f64 * fog) as u8;
            buffer[idx + 2] = (base.2 as f64 * fog) as u8;

            floor_x += floor_step_x;
            floor_y += floor_step_y;
        }
    }

    let _ = texture.update(None, buffer, pitch);
    let _ = canvas.copy(texture, None, Some(Rect::new(0, horizon, w as u32, floor_h as u32)));
}

/// Dibuja las paredes y llena `z_buffer` con la distancia perpendicular de
/// cada columna — el sprite del enemigo (render::sprites) lo usa para saber
/// qué columnas están tapadas por una pared más cercana.
fn render_walls(canvas: &mut Canvas<Window>, player: &Player, z_buffer: &mut [f64], w: i32, h: i32) {
    let view = camera_view(player);

    for x in 0..w {
        // camera_x barre de -1 (borde izq.) a 1 (borde der.) de la pantalla.
        let camera_x = 2.0 * (x as f64) / (w as f64) - 1.0;
        let ray_dir_x = view.dir_x + view.plane_x * camera_x;
        let ray_dir_y = view.dir_y + view.plane_y * camera_x;

        let hit: Option<RayHit> = cast_ray(player.x, player.y, ray_dir_x, ray_dir_y);
        let Some(hit) = hit else {
            z_buffer[x as usize] = f64::INFINITY;
            continue;
        };
        z_buffer[x as usize] = hit.perp_dist;

        let line_height = (h as f64 / hit.perp_dist) as i32;
        let draw_start = (-line_height / 2 + h / 2).max(0);
        let draw_end = (line_height / 2 + h / 2).min(h - 1);
        let total_h = (draw_end - draw_start).max(1);

        let color = shaded_color(hit.zone, hit.side, hit.perp_dist);
        canvas.set_draw_color(color);
        let _ = canvas.fill_rect(Rect::new(x, draw_start, 1, total_h as u32));

        // Zócalo: ancla la pared al piso con una franja más oscura, como en un
        // cuarto real, en vez de flotar como un plano liso.
        if total_h > 10 {
            let base_h = ((total_h as f64) * 0.08).ceil().max(1.0) as i32;
            canvas.set_draw_color(darken(color, BASEBOARD_FACTOR));
            let _ = canvas.fill_rect(Rect::new(x, draw_end - base_h, 1, base_h as u32));
        }
    }
}

pub fn render_scene(
    canvas: &mut Canvas<Window>,
    floor_texture: &mut Texture,
    floor_buffer: &mut [u8],
    z_buffer: &mut [f64],
    player: &Player,
    screen_w: u32,
    screen_h: u32,
) {
    let w = screen_w as i32;
    let h = screen_h as i32;

    render_ceiling(canvas, w, h);
    render_floor(canvas, floor_texture, floor_buffer, player, w, h);
    render_walls(canvas, player, z_buffer, w, h);
}
