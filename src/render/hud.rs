//! HUD: contador de FPS (15 pts), minimapa (10 pts) y vida en corazones.

use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};

use super::text::{draw_text, draw_text_centered};
use crate::engine::minimap;
use crate::entities::player::{Player, MAX_LIFE};
use crate::map::{BLOCK_GRID, BLOCK_SIZE};

const FPS_COLOR: Color = Color::RGB(120, 230, 140);
const MINIMAP_BG: Color = Color::RGBA(12, 10, 14, 210);
const MINIMAP_WALL: Color = Color::RGB(210, 200, 190);
const MINIMAP_PLAYER: Color = Color::RGB(225, 60, 60);

/// Ventana móvil corta para promediar el FPS y que no "tiemble" frame a frame.
pub struct FpsCounter {
    samples: Vec<f64>,
    pub current: f64,
}

impl FpsCounter {
    pub fn new() -> Self {
        FpsCounter { samples: Vec::with_capacity(30), current: 0.0 }
    }

    pub fn push_frame_time(&mut self, dt_secs: f64) {
        // Umbral mínimo: un dt casi cero (posible en el primer frame, antes de
        // que el timing se estabilice) da un FPS absurdamente inflado que
        // luego contamina el promedio móvil durante ~20 frames.
        if dt_secs < 0.001 {
            return;
        }
        self.samples.push(1.0 / dt_secs);
        if self.samples.len() > 20 {
            self.samples.remove(0);
        }
        self.current = self.samples.iter().sum::<f64>() / self.samples.len() as f64;
    }

    pub fn draw(
        &self,
        canvas: &mut Canvas<Window>,
        texture_creator: &TextureCreator<WindowContext>,
        font: &Font,
    ) {
        let text = format!("FPS: {:.0}", self.current);
        draw_text(canvas, texture_creator, font, &text, 10, 10, FPS_COLOR);
    }
}

// --- Corazones de vida (ver SKILL.md, "Vida del jugador — corazones") -----
//
// Pixel art crudo a propósito, definido por código (bitmap 7x7 booleano,
// mismo espíritu que un bitmap font) en vez de un PNG — no hace falta arte de
// Cami para esto y funciona como estilo final, no solo placeholder.

const HEART_W: usize = 7;
const HEART_H: usize = 7;
const HEART_CELL_PX: i32 = 4;
const HEART_GAP_PX: i32 = 8;
const HEARTS_TOTAL: u32 = 5;
const LIFE_PER_HEART: f64 = MAX_LIFE / HEARTS_TOTAL as f64;

const HEART_COLOR_FULL: Color = Color::RGB(90, 10, 22);
const HEART_COLOR_CRACKED: Color = Color::RGB(55, 12, 15);

#[rustfmt::skip]
const HEART_FULL: [[u8; HEART_W]; HEART_H] = [
    [0,1,1,0,1,1,0],
    [1,1,1,1,1,1,1],
    [1,1,1,1,1,1,1],
    [1,1,1,1,1,1,1],
    [0,1,1,1,1,1,0],
    [0,0,1,1,1,0,0],
    [0,0,0,1,0,0,0],
];

// Mismo contorno que HEART_FULL pero con una grieta tallada — se usa cuando al
// corazón le queda menos de la mitad de sus puntos de vida.
#[rustfmt::skip]
const HEART_CRACKED: [[u8; HEART_W]; HEART_H] = [
    [0,1,1,0,1,1,0],
    [1,1,1,1,0,1,1],
    [1,1,1,0,1,1,1],
    [1,1,0,1,1,1,1],
    [0,1,1,1,1,1,0],
    [0,0,1,0,1,0,0],
    [0,0,0,1,0,0,0],
];

fn draw_heart_bitmap(canvas: &mut Canvas<Window>, bitmap: &[[u8; HEART_W]; HEART_H], x: i32, y: i32, color: Color) {
    draw_heart_icon(canvas, bitmap, x, y, HEART_CELL_PX, color);
}

/// Dibuja un corazón suelto (mismo bitmap que los de la vida) en cualquier
/// posición/tamaño — usado, por ejemplo, junto al prompt de inicio de la
/// pantalla de bienvenida.
pub fn draw_heart_icon(
    canvas: &mut Canvas<Window>,
    bitmap: &[[u8; HEART_W]; HEART_H],
    x: i32,
    y: i32,
    cell_px: i32,
    color: Color,
) {
    canvas.set_draw_color(color);
    for (row, cells) in bitmap.iter().enumerate() {
        for (col, &on) in cells.iter().enumerate() {
            if on == 0 {
                continue;
            }
            let px = x + col as i32 * cell_px;
            let py = y + row as i32 * cell_px;
            let _ = canvas.fill_rect(Rect::new(px, py, cell_px as u32, cell_px as u32));
        }
    }
}

/// Vida del jugador como 5 corazones, debajo del contador de FPS. Cada corazón
/// vale 20 de vida: entero mientras le quede más de la mitad de sus puntos,
/// agrietado por debajo de la mitad, desaparece en 0. El umbral de
/// "enfurecido" del enemigo (≤50% del total) es independiente de esto — pero
/// coincide con que el 3er corazón (de 5) empiece a agrietarse, así que la
/// lectura visual y el peligro real quedan alineados.
pub fn draw_hearts(canvas: &mut Canvas<Window>, player: &Player, x: i32, y: i32) {
    let heart_px_w = HEART_W as i32 * HEART_CELL_PX;
    for i in 0..HEARTS_TOTAL {
        let heart_life = (player.life - i as f64 * LIFE_PER_HEART).clamp(0.0, LIFE_PER_HEART);
        if heart_life <= 0.0 {
            continue;
        }
        let hx = x + i as i32 * (heart_px_w + HEART_GAP_PX);
        if heart_life > LIFE_PER_HEART / 2.0 {
            draw_heart_bitmap(canvas, &HEART_FULL, hx, y, HEART_COLOR_FULL);
        } else {
            draw_heart_bitmap(canvas, &HEART_CRACKED, hx, y, HEART_COLOR_CRACKED);
        }
    }
}

const TIMER_COLOR: Color = Color::RGB(210, 205, 195);
const TIMER_COLOR_LOW: Color = Color::RGB(226, 20, 20); // últimos 10s — mismo rojo de marca.
const TIMER_LOW_THRESHOLD: f64 = 10.0;

/// Cuenta regresiva de supervivencia, centrada arriba de la pantalla —
/// `M:SS`. Se pone roja en los últimos 10 segundos como aviso.
pub fn draw_timer(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    font: &Font,
    remaining_secs: f64,
    center_x: i32,
    y: i32,
) {
    let secs_total = remaining_secs.max(0.0).ceil() as i32;
    let minutes = secs_total / 60;
    let seconds = secs_total % 60;
    let text = format!("{minutes}:{seconds:02}");
    let color = if remaining_secs <= TIMER_LOW_THRESHOLD { TIMER_COLOR_LOW } else { TIMER_COLOR };
    draw_text_centered(canvas, texture_creator, font, &text, center_x, y, color);
}

/// Dibuja el minimapa (paredes del grid + marcador de jugador con su orientación)
/// anclado en la esquina superior derecha de la pantalla.
pub fn draw_minimap(canvas: &mut Canvas<Window>, player: &Player, screen_w: i32) {
    let layout = minimap::layout(screen_w);
    let cell = layout.cell_px;

    // Fondo semitransparente para que el minimapa se lea sobre cualquier escena.
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    canvas.set_draw_color(MINIMAP_BG);
    let _ = canvas.fill_rect(Rect::new(
        layout.origin_x - 4,
        layout.origin_y - 4,
        (layout.size_px + 8) as u32,
        (layout.size_px + 8) as u32,
    ));

    canvas.set_draw_color(MINIMAP_WALL);
    for (row, cells) in BLOCK_GRID.iter().enumerate().take(BLOCK_SIZE) {
        for (col, &solid) in cells.iter().enumerate().take(BLOCK_SIZE) {
            if !solid {
                continue;
            }
            let x0 = layout.origin_x + col as i32 * cell;
            let y0 = layout.origin_y + row as i32 * cell;
            let _ = canvas.fill_rect(Rect::new(x0, y0, cell as u32, cell as u32));
        }
    }

    let (px, py) = minimap::player_marker_px(&layout, player);
    canvas.set_draw_color(MINIMAP_PLAYER);
    let _ = canvas.fill_rect(Rect::new(px - 2, py - 2, 4, 4));

    // Línea corta indicando hacia dónde mira la cámara.
    let (dir_x, dir_y) = player.dir();
    let tip = Point::new(px + (dir_x * 7.0) as i32, py + (dir_y * 7.0) as i32);
    let _ = canvas.draw_line(Point::new(px, py), tip);
}
