//! Pantalla de éxito: se muestra al sobrevivir el tiempo de la dificultad
//! elegida. Fondo `assets/ui/success_background.png` (dibujo de Cami) + título
//! "I thought you loved me" (mismo tratamiento que el título principal) +
//! "YOU ESCAPED!".

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};

use crate::render::text::{draw_text_centered, draw_text_centered_scaled, draw_title_stretched};

const TITLE_COLOR: (u8, u8, u8) = (210, 205, 215);

pub struct SuccessState {
    clock: f64,
}

impl SuccessState {
    pub fn new() -> Self {
        SuccessState { clock: 0.0 }
    }

    pub fn update(&mut self, dt: f64) {
        self.clock += dt;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        canvas: &mut Canvas<Window>,
        texture_creator: &TextureCreator<WindowContext>,
        title_font: &Font,
        body_font: &Font,
        background: &Texture,
        w: i32,
        h: i32,
    ) {
        let _ = canvas.copy(background, None, Some(Rect::new(0, 0, w as u32, h as u32)));

        let flash = 0.6 + 0.4 * (0.5 + 0.5 * (self.clock * 1.8).sin());
        let title_color =
            Color::RGBA(TITLE_COLOR.0, TITLE_COLOR.1, TITLE_COLOR.2, (flash * 255.0) as u8);
        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        // Centrado sobre el medio vertical de la pantalla (antes quedaba muy
        // arriba); el gap con "YOU ESCAPED!" se acorta metiéndolo debajo del
        // título sin padding de sobra, mismo criterio que la bienvenida.
        let title_y = (h as f64 * 0.36) as i32;
        let title_h = draw_title_stretched(
            canvas,
            texture_creator,
            title_font,
            "I THOUGHT YOU LOVED ME",
            w / 2,
            title_y,
            title_color,
            1.7,
            (w as f64 * 0.92) as i32,
        );

        draw_text_centered(
            canvas,
            texture_creator,
            body_font,
            "YOU ESCAPED!",
            w / 2,
            title_y + title_h as i32 - (title_h as f64 * 0.15) as i32,
            Color::RGB(200, 195, 190),
        );

        draw_text_centered_scaled(
            canvas,
            texture_creator,
            body_font,
            "press ENTER to return to the menu",
            w / 2,
            h - 60,
            Color::RGB(150, 145, 140),
            0.75,
        );
    }
}
