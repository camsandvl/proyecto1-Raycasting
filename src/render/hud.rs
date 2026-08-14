//! HUD: contador de FPS (15 pts) y minimapa (10 pts).

use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};

use super::text::draw_text;
use crate::engine::minimap;
use crate::entities::player::Player;
use crate::map::{BLOCK_GRID, BLOCK_SIZE};

const FPS_COLOR: Color = Color::RGB(120, 230, 140);
const LIFE_COLOR_OK: Color = Color::RGB(120, 230, 140);
const LIFE_COLOR_LOW: Color = Color::RGB(235, 70, 70);
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

/// Vida del jugador, debajo del contador de FPS — cambia de color al cruzar el
/// mismo umbral (≤50%) que dispara el estado "enfurecido" del enemigo, así se
/// puede confirmar visualmente que el umbral dispara en el momento correcto.
pub fn draw_life(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    font: &Font,
    player: &Player,
) {
    let color = if player.life_below_half() { LIFE_COLOR_LOW } else { LIFE_COLOR_OK };
    let text = format!("VIDA: {:.0}", player.life);
    draw_text(canvas, texture_creator, font, &text, 10, 40, color);
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
