//! Dibuja la escena 3D: techo (degradado), piso (floor-casting con perspectiva
//! real, para que se vea como un plano que retrocede y no un rectángulo plano) y
//! paredes columna por columna a partir de los impactos del DDA.
//!
//! Paredes: Recibidor/Dormitorio/Sala muestrean una textura real (columna de 1px
//! de ancho de `assets/textures/wallpaper.png`, escalada por GPU al alto de la
//! franja) usando `RayHit::wall_x` como coordenada U — el sombreado (lado E/W
//! más oscuro, niebla por distancia) se aplica con `Texture::set_color_mod`
//! (tinte multiplicativo) en vez de tocar píxeles a mano, así un solo
//! `canvas.copy` por columna alcanza. La Cocina sigue con color plano generado
//! por código (no tiene textura de Cami, ver SKILL.md).
//!
//! `wallpaper.png` es un placeholder compartido por las 3 zonas hasta que Cami
//! entregue las 3 texturas finales, una por zona.
//!
//! Piso: ya no es un checker de dos tonos — muestrea `assets/textures/floor.png`
//! por píxel (vía `PixelTexture`, acceso a bytes crudos) directo en el mismo
//! buffer del floor-casting, una vez por celda de mundo (1×1, tileable).
//! Techo: sigue siendo degradado liso por código (`CEILING_TOP`/`CEILING_HORIZON`),
//! no textura — cambiar esos dos colores alcanza para recolorearlo.

use sdl2::image::LoadSurface;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::surface::Surface;
use sdl2::video::Window;

use crate::engine::camera::camera_view;
use crate::engine::raycasting::{cast_ray, RayHit, Side};
use crate::entities::player::Player;
use crate::map::Zone;

// Techo — degradado liso, casi negro con un dejo amarillento a pedido de Cami.
pub const CEILING_TOP: (u8, u8, u8) = (3, 2, 0);
pub const CEILING_HORIZON: (u8, u8, u8) = (12, 10, 3);

/// Textura leída a memoria de CPU (RGB24 sin padding) para poder muestrear
/// píxel a píxel dentro del loop de floor-casting, en vez de un `Texture` de
/// GPU (que no se puede leer directo). Se carga una sola vez al arrancar.
pub struct PixelTexture {
    w: i32,
    h: i32,
    pixels: Vec<u8>,
}

impl PixelTexture {
    pub fn load(path: &str) -> Result<Self, String> {
        let surface = Surface::from_file(path)?;
        let surface = surface.convert_format(PixelFormatEnum::RGB24)?;
        let w = surface.width() as i32;
        let h = surface.height() as i32;
        let pitch = surface.pitch() as usize;
        let row_bytes = w as usize * 3;
        let mut pixels = vec![0u8; row_bytes * h as usize];
        surface.with_lock(|src| {
            for y in 0..h as usize {
                let src_row = &src[y * pitch..y * pitch + row_bytes];
                let dst_row = &mut pixels[y * row_bytes..(y + 1) * row_bytes];
                dst_row.copy_from_slice(src_row);
            }
        });
        Ok(PixelTexture { w, h, pixels })
    }

    /// Muestrea el color en coordenadas `u`, `v` (se envuelven a 0.0-1.0 solas,
    /// no hace falta que el llamador las recorte — así una celda de mundo
    /// cualquiera, positiva o negativa, tilea sin costuras).
    #[inline]
    fn sample(&self, u: f64, v: f64) -> (u8, u8, u8) {
        let uu = u.rem_euclid(1.0);
        let vv = v.rem_euclid(1.0);
        let tx = ((uu * self.w as f64) as i32).clamp(0, self.w - 1) as usize;
        let ty = ((vv * self.h as f64) as i32).clamp(0, self.h - 1) as usize;
        let idx = (ty * self.w as usize + tx) * 3;
        (self.pixels[idx], self.pixels[idx + 1], self.pixels[idx + 2])
    }
}

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
fn render_floor(
    canvas: &mut Canvas<Window>,
    texture: &mut Texture,
    buffer: &mut [u8],
    floor_tex: &PixelTexture,
    player: &Player,
    w: i32,
    h: i32,
) {
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
            let base = floor_tex.sample(floor_x, floor_y);

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
fn render_walls(canvas: &mut Canvas<Window>, wall_texture: &mut Texture, player: &Player, z_buffer: &mut [f64], w: i32, h: i32) {
    let view = camera_view(player);
    let tex_query = wall_texture.query();
    let tex_w = tex_query.width as i32;
    let tex_h = tex_query.height as i32;

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

        if hit.zone == Zone::Cocina {
            // Sin textura de Cami — se queda con el color plano generado por código.
            let color = shaded_color(hit.zone, hit.side, hit.perp_dist);
            canvas.set_draw_color(color);
            let _ = canvas.fill_rect(Rect::new(x, draw_start, 1, total_h as u32));
            continue;
        }

        // Textura real: una tira de 1px de ancho de la columna `wall_x` de la
        // textura, escalada por GPU al alto de la franja. El sombreado (lado
        // E/W + niebla por distancia) se aplica como tinte multiplicativo, no
        // tocando píxeles — barato y sigue siendo un solo `canvas.copy`.
        let side_factor = if hit.side == Side::Horizontal { 0.65 } else { 1.0 };
        let fog_factor = (1.0 - (hit.perp_dist / 14.0).min(0.55)).max(0.45);
        let shade = ((side_factor * fog_factor).clamp(0.0, 1.0) * 255.0) as u8;

        let src_x = ((hit.wall_x * tex_w as f64) as i32).clamp(0, tex_w - 1);
        let src = Rect::new(src_x, 0, 1, tex_h as u32);
        let dest = Rect::new(x, draw_start, 1, total_h as u32);
        wall_texture.set_color_mod(shade, shade, shade);
        let _ = canvas.copy(wall_texture, Some(src), Some(dest));
    }
}

pub fn render_scene(
    canvas: &mut Canvas<Window>,
    floor_texture: &mut Texture,
    floor_buffer: &mut [u8],
    floor_tex: &PixelTexture,
    wall_texture: &mut Texture,
    z_buffer: &mut [f64],
    player: &Player,
    screen_w: u32,
    screen_h: u32,
) {
    let w = screen_w as i32;
    let h = screen_h as i32;

    render_ceiling(canvas, w, h);
    render_floor(canvas, floor_texture, floor_buffer, floor_tex, player, w, h);
    render_walls(canvas, wall_texture, player, z_buffer, w, h);
}
