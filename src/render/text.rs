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

/// Igual que `draw_text_centered` pero encogido/agrandado uniformemente por
/// `scale` (1.0 = tamaño normal de la fuente) — para textos secundarios que
/// necesitan verse más chicos que el resto sin cargar una fuente aparte.
#[allow(clippy::too_many_arguments)]
pub fn draw_text_centered_scaled(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    font: &Font,
    text: &str,
    center_x: i32,
    y: i32,
    color: Color,
    scale: f64,
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
    let dest_w = ((width as f64) * scale).max(1.0) as u32;
    let dest_h = ((height as f64) * scale).max(1.0) as u32;
    let x = center_x - (dest_w as i32) / 2;
    let _ = canvas.copy(&texture, None, Some(Rect::new(x, y, dest_w, dest_h)));
}

/// Ancho/alto naturales que ocuparía `text` con `font`, sin dibujar nada — para
/// calcular layout (ej. posicionar algo debajo de un título) antes de dibujar.
pub fn measure_text(font: &Font, text: &str) -> (u32, u32) {
    font.size_of(text).unwrap_or((0, 0))
}

/// Título "elongado": dibuja `text` centrado horizontalmente, estirado
/// verticalmente por `vertical_scale` (1.0 = normal, >1.0 = letras más largas),
/// y encogido (ancho Y alto por igual) si el texto natural no entraría en
/// `max_width` — así un título largo ("I thought you loved me") no se corta
/// en los bordes de la ventana. Ver SKILL.md, nota de implementación "Texto
/// estirado verticalmente" — no depende de encontrar una fuente ya alta,
/// cualquier fuente sirve con esta técnica porque solo se altera el rect de
/// destino, no la fuente en sí.
/// Devuelve el alto final dibujado (útil para posicionar lo que va debajo).
#[allow(clippy::too_many_arguments)]
pub fn draw_title_stretched(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    font: &Font,
    text: &str,
    center_x: i32,
    y: i32,
    color: Color,
    vertical_scale: f64,
    max_width: i32,
) -> u32 {
    if text.is_empty() {
        return 0;
    }
    let surface = match font.render(text).blended(color) {
        Ok(s) => s,
        Err(_) => return 0,
    };
    let texture = match texture_creator.create_texture_from_surface(&surface) {
        Ok(t) => t,
        Err(_) => return 0,
    };
    let sdl2::render::TextureQuery { width, height, .. } = texture.query();
    let fit_scale = if width as i32 > max_width && width > 0 {
        max_width as f64 / width as f64
    } else {
        1.0
    };
    let dest_w = ((width as f64) * fit_scale).max(1.0) as u32;
    let dest_h = ((height as f64) * vertical_scale * fit_scale).max(1.0) as u32;
    let x = center_x - (dest_w as i32) / 2;
    let _ = canvas.copy(&texture, None, Some(Rect::new(x, y, dest_w, dest_h)));
    dest_h
}
