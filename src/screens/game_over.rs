//! Pantalla de game over: close-up de la cara de Erica de fondo
//! (`assets/ui/jumpscare_face.png`, dibujo de Cami, pantalla completa) +
//! overlay oscuro + "Happy anniversary!" parpadeando + "GAME OVER - press R to
//! retry" (mismo tamaño/posición que el prompt de volver al menú de la
//! pantalla de éxito).

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Canvas, Texture, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};

use crate::render::text::{draw_text_centered_scaled, draw_title_stretched};

const OVERLAY_ALPHA: u8 = 110;
const TITLE_COLOR: (u8, u8, u8) = (0xe2, 0x14, 0x14); // rojo de marca (ver welcome.rs)

pub struct GameOverState {
    clock: f64,
}

impl GameOverState {
    pub fn new() -> Self {
        GameOverState { clock: 0.0 }
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
        jumpscare_face: &Texture,
        w: i32,
        h: i32,
    ) {
        let _ = canvas.copy(jumpscare_face, None, Some(Rect::new(0, 0, w as u32, h as u32)));

        // Overlay oscuro semi-transparente sobre todo (ver SKILL.md).
        canvas.set_blend_mode(BlendMode::Blend);
        canvas.set_draw_color(Color::RGBA(0, 0, 0, OVERLAY_ALPHA));
        let _ = canvas.fill_rect(Rect::new(0, 0, w as u32, h as u32));

        let flash = 0.55 + 0.45 * (0.5 + 0.5 * (self.clock * 3.0).sin());
        let title_color =
            Color::RGBA(TITLE_COLOR.0, TITLE_COLOR.1, TITLE_COLOR.2, (flash * 255.0) as u8);
        canvas.set_blend_mode(BlendMode::Blend);
        draw_title_stretched(
            canvas,
            texture_creator,
            title_font,
            "HAPPY ANNIVERSARY!",
            w / 2,
            (h as f64 * 0.36) as i32,
            title_color,
            1.4,
            (w as f64 * 0.92) as i32,
        );

        draw_text_centered_scaled(
            canvas,
            texture_creator,
            body_font,
            "GAME OVER - press R to retry",
            w / 2,
            h - 60,
            Color::RGB(150, 145, 140),
            0.75,
        );
    }
}
