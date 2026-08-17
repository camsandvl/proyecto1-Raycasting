//! Pantalla de bienvenida: título "Not your house" parpadeando y estirado
//! verticalmente, luego (al presionar ENTER) el selector de dificultad — todo
//! por teclado (ver SKILL.md, "Historia, título y pantallas narrativas").
//! Paleta y tipografías definidas por Cami: rojo (#e21414), título en
//! `Instrument Serif`, UI en `Lora`.
//!
//! Fondo, dibujo de Cami: `title_background.png` (wallpaper floral),
//! compartido por las dos etapas (título y selector de dificultad).

use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Canvas, Texture, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};

use crate::difficulty::Difficulty;
use crate::render::text::{draw_text, draw_text_centered, draw_title_stretched, measure_text};

const RED: Color = Color::RGB(0xe2, 0x14, 0x14);
const DIFF_IDLE: Color = Color::RGB(0x8a, 0x55, 0x45);

/// Rojo oscurecido para el extremo "apagado" del parpadeo. El parpadeo se
/// hace variando el brillo del color a alpha 255 fijo, no la transparencia —
/// sobre el fondo crema, bajar el alpha del rojo lo mezcla con el fondo y se
/// ve anaranjado/lavado en vez de un rojo más oscuro.
const RED_DIM: Color = Color::RGB(0x6a, 0x0c, 0x0c);

fn pulse_red(t: f64) -> Color {
    let f = t.clamp(0.0, 1.0);
    Color::RGB(
        lerp_u8(RED_DIM.r, RED.r, f),
        lerp_u8(RED_DIM.g, RED.g, f),
        lerp_u8(RED_DIM.b, RED.b, f),
    )
}

fn lerp_u8(a: u8, b: u8, t: f64) -> u8 {
    (a as f64 + (b as f64 - a as f64) * t.clamp(0.0, 1.0)) as u8
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Title,
    Difficulty,
}

pub struct WelcomeState {
    pub difficulty: Difficulty,
    stage: Stage,
    clock: f64,
}

impl WelcomeState {
    pub fn new() -> Self {
        WelcomeState { difficulty: Difficulty::Medium, stage: Stage::Title, clock: 0.0 }
    }

    /// Procesa las teclas presionadas ESTE frame (edge-triggered). Devuelve
    /// `true` solo cuando el jugador confirmó la dificultad y hay que pasar a
    /// la cinemática — el ENTER de la pantalla de título solo avanza de etapa.
    pub fn handle_keydowns(&mut self, keydowns: &[Scancode]) -> bool {
        match self.stage {
            Stage::Title => {
                if keydowns.iter().any(is_confirm) {
                    self.stage = Stage::Difficulty;
                }
                false
            }
            Stage::Difficulty => {
                for &sc in keydowns {
                    match sc {
                        Scancode::Num1 => self.difficulty = Difficulty::Easy,
                        Scancode::Num2 => self.difficulty = Difficulty::Medium,
                        Scancode::Num3 => self.difficulty = Difficulty::Hard,
                        _ if is_confirm(&sc) => return true,
                        _ => {}
                    }
                }
                false
            }
        }
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
        ui_font: &Font,
        title_background: &Texture,
        difficulty_background: &Texture,
        w: i32,
        h: i32,
    ) {
        canvas.set_blend_mode(BlendMode::Blend);

        match self.stage {
            Stage::Title => {
                let _ = canvas.copy(title_background, None, Some(Rect::new(0, 0, w as u32, h as u32)));
                self.draw_title_stage(canvas, texture_creator, title_font, ui_font, w, h);
            }
            Stage::Difficulty => {
                let _ = canvas.copy(difficulty_background, None, Some(Rect::new(0, 0, w as u32, h as u32)));
                self.draw_difficulty_stage(canvas, texture_creator, ui_font, w, h);
            }
        }
    }

    fn draw_title_stage(
        &self,
        canvas: &mut Canvas<Window>,
        texture_creator: &TextureCreator<WindowContext>,
        title_font: &Font,
        ui_font: &Font,
        w: i32,
        _h: i32,
    ) {
        let flash = 0.55 + 0.45 * (0.5 + 0.5 * (self.clock * 2.2).sin());
        let title_color = pulse_red(flash);
        // Casi sin padding a los costados, como pidió Cami — el auto-ajuste de
        // ancho de draw_title_stretched hace que cada línea llene ~97% del
        // ancho de la ventana (la fuente se carga bien grande en main.rs para
        // asegurar que el ancho natural siempre supere este límite). El alto
        // lo controla `vscale` — con el ancho ya fijo por el auto-ajuste, subir
        // este número solo agranda verticalmente, no desborda a los costados.
        let max_w = (w as f64 * 0.97) as i32;
        let vscale = 1.95;

        let line1_y = 2;
        let line1_h =
            draw_title_stretched(canvas, texture_creator, title_font, "NOT YOUR", w / 2, line1_y, title_color, vscale, max_w);
        let line2_y = line1_y + line1_h as i32 - (line1_h as f64 * 0.4) as i32;
        let line2_h =
            draw_title_stretched(canvas, texture_creator, title_font, "HOUSE", w / 2, line2_y, title_color, vscale, max_w);

        // Pegado al título a propósito — sin padding de sobra entre el título
        // y el prompt, tal como pidió Cami.
        let prompt_flash = 0.6 + 0.4 * (0.5 + 0.5 * (self.clock * 3.0).sin());
        let prompt_color = pulse_red(prompt_flash);
        let text = "press enter to start";
        let prompt_y = line2_y + line2_h as i32 - (line2_h as f64 * 0.22) as i32;

        draw_text_centered(canvas, texture_creator, ui_font, text, w / 2, prompt_y, prompt_color);
    }

    fn draw_difficulty_stage(
        &self,
        canvas: &mut Canvas<Window>,
        texture_creator: &TextureCreator<WindowContext>,
        ui_font: &Font,
        w: i32,
        h: i32,
    ) {
        draw_text_centered(canvas, texture_creator, ui_font, "choose your difficulty", w / 2, (h as f64 * 0.34) as i32, RED);

        let seconds = [90, 150, 210];
        let options = [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard];
        let labels: Vec<String> =
            options.iter().zip(seconds).map(|(d, s)| format!("{} ({}s)", d.label(), s)).collect();
        let gap = 44;
        let widths: Vec<u32> = labels.iter().map(|label| measure_text(ui_font, label).0).collect();
        let total_w: i32 = widths.iter().map(|w| *w as i32).sum::<i32>() + gap * (options.len() as i32 - 1);
        let mut x = w / 2 - total_w / 2;
        let y = (h as f64 * 0.46) as i32;
        for (i, diff) in options.iter().enumerate() {
            let color = if *diff == self.difficulty { RED } else { DIFF_IDLE };
            draw_text(canvas, texture_creator, ui_font, &labels[i], x, y, color);
            x += widths[i] as i32 + gap;
        }

        draw_text_centered(canvas, texture_creator, ui_font, "1 / 2 / 3 to choose", w / 2, y + 46, DIFF_IDLE);

        let hint_flash = 0.6 + 0.4 * (0.5 + 0.5 * (self.clock * 3.0).sin());
        let hint_color = pulse_red(hint_flash);
        draw_text_centered(canvas, texture_creator, ui_font, "press enter to begin", w / 2, h - (h as f64 * 0.18) as i32, hint_color);
    }
}

fn is_confirm(sc: &Scancode) -> bool {
    matches!(sc, Scancode::Return | Scancode::Return2 | Scancode::KpEnter)
}
