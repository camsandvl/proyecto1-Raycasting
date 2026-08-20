//! Cinemática de introducción: fondo estático del cuarto (dibujo de Cami,
//! `assets/ui/intro_backdrop.png` — incluye la figura oscura ya dibujada
//! dentro de la ilustración, no se anima por código) + 3 líneas de texto
//! secuenciales encima. Sin video real — ver nota técnica en SKILL.md ("por
//! qué no hay video real").

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};

use crate::render::text::draw_text_centered;

const LINES: [&str; 3] = [
    "It's your anniversary!",
    "Your girlfriend Erica has been acting weird lately...",
    "Happy anniversary.",
];

const LINE_DURATION: f64 = 2.6;
const FADE_DURATION: f64 = 0.5;
const TOTAL_DURATION: f64 = LINE_DURATION * LINES.len() as f64;

pub struct IntroState {
    elapsed: f64,
}

impl IntroState {
    pub fn new() -> Self {
        IntroState { elapsed: 0.0 }
    }

    /// Devuelve `true` cuando la cinemática terminó y hay que pasar al gameplay.
    pub fn update(&mut self, dt: f64) -> bool {
        self.elapsed += dt;
        self.elapsed >= TOTAL_DURATION
    }

    pub fn draw(
        &self,
        canvas: &mut Canvas<Window>,
        texture_creator: &TextureCreator<WindowContext>,
        body_font: &Font,
        backdrop: &Texture,
        w: i32,
        h: i32,
    ) {
        let _ = canvas.copy(backdrop, None, Some(Rect::new(0, 0, w as u32, h as u32)));

        let line_idx = ((self.elapsed / LINE_DURATION) as usize).min(LINES.len() - 1);
        let t_in_line = self.elapsed - line_idx as f64 * LINE_DURATION;
        let alpha = line_alpha(t_in_line);
        if alpha > 0.01 {
            canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
            let color = Color::RGBA(225, 220, 215, (alpha * 255.0) as u8);
            draw_text_centered(canvas, texture_creator, body_font, LINES[line_idx], w / 2, h / 2, color);
        }
    }
}

/// Fade in/out dentro de una línea: sube en `FADE_DURATION`, se mantiene, baja
/// en los últimos `FADE_DURATION` segundos.
fn line_alpha(t: f64) -> f64 {
    if t < FADE_DURATION {
        t / FADE_DURATION
    } else if t > LINE_DURATION - FADE_DURATION {
        ((LINE_DURATION - t) / FADE_DURATION).max(0.0)
    } else {
        1.0
    }
}
