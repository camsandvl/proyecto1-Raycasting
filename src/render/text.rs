//! Utilidad delgada sobre `sdl2::ttf` para dibujar texto. Se usa para el HUD
//! (contador de FPS, vida, timer) y, más adelante, para las pantallas de
//! bienvenida/éxito/game over (ver assets/fonts/CREDITS.md para las fuentes).

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};

/// Dibuja `text` con la esquina superior izquierda en (x, y). Silenciosamente no
/// dibuja nada si el string está vacío (sdl2_ttf no acepta render de texto vacío).
pub fn draw_text(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    font: &Font,
    text: &str,
    x: i32,
    y: i32,
    color: Color,
) {
    if text.is_empty() {
        return;
    }
    let surface = match font.render(text).blended(color) {
        Ok(s) => s,
        Err(_) => return,
    };
    let texture = match texture_creator.create_texture_from_surface(&surface) {
        Ok(t) => t,
        Err(_) => return,
    };
    let sdl2::render::TextureQuery { width, height, .. } = texture.query();
    let _ = canvas.copy(&texture, None, Some(Rect::new(x, y, width, height)));
}

/// Igual que `draw_text` pero centrado horizontalmente sobre `center_x`.
pub fn draw_text_centered(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    font: &Font,
    text: &str,
    center_x: i32,
    y: i32,
    color: Color,
) {
    if text.is_empty() {
        return;
    }
    let surface = match font.render(text).blended(color) {
        Ok(s) => s,
        Err(_) => return,
    };
    let texture = match texture_creator.create_texture_from_surface(&surface) {
        Ok(t) => t,
        Err(_) => return,
    };
    let sdl2::render::TextureQuery { width, height, .. } = texture.query();
    let x = center_x - (width as i32) / 2;
    let _ = canvas.copy(&texture, None, Some(Rect::new(x, y, width, height)));
}
