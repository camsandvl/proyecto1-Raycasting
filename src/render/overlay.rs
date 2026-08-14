//! Overlays de pantalla completa alpha-blended. Usado ahora para el aviso de
//! peligro (contacto con el enemigo); Fase 6 reutiliza `fill_screen` para el
//! overlay oscuro semi-transparente de la pantalla de game over.

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;

/// Pinta un rectángulo semitransparente sobre toda la pantalla.
pub fn fill_screen(canvas: &mut Canvas<Window>, color: Color, alpha: u8, w: i32, h: i32) {
    if alpha == 0 {
        return;
    }
    canvas.set_blend_mode(BlendMode::Blend);
    canvas.set_draw_color(Color::RGBA(color.r, color.g, color.b, alpha));
    let _ = canvas.fill_rect(Rect::new(0, 0, w as u32, h as u32));
}
