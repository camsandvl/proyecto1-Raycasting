//! "Not your house" — Ray Caster de terror. Ver SKILL.md para el diseño completo.
//!
//! Máquina de estados de alto nivel: Welcome → Intro → Playing → Success | GameOver
//! (y de vuelta a Welcome). Ver `screens/` para cada pantalla y SKILL.md,
//! sección "Historia, título y pantallas narrativas", para el guion completo.

mod audio;
mod difficulty;
mod engine;
mod entities;
mod map;
mod render;
mod screens;

use std::time::Instant;

use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

use audio::mixer::AudioEngine;
use difficulty::Difficulty;
use entities::enemy::{Enemy, CONTACT_RANGE, DAMAGE_PER_SECOND};
use entities::player::Player;
use map::{Map, ENEMY_SPAWN_CELL, PLAYER_SPAWN_CELL};
use render::hud::FpsCounter;
use screens::game_over::GameOverState;
use screens::intro::IntroState;
use screens::success::SuccessState;
use screens::welcome::WelcomeState;

const SCREEN_WIDTH: u32 = 960;
const SCREEN_HEIGHT: u32 = 600;

/// Cuánto se muestra el aviso pasajero al entrar al gameplay (ver SKILL.md).
const START_LABEL_DURATION: f64 = 4.5;
const START_LABEL_FADE: f64 = 0.7;
const START_LABEL_TEXT: &str = "Help is coming. You just have to hold on a little longer.";

/// Segundo aviso, secuenciado justo después de que el primero termina de
/// desvanecerse (arranca cuando `house_label_timer` entra en su propia
/// ventana de duración — ver RunState::new).
const HOUSE_LABEL_DELAY: f64 = START_LABEL_DURATION;
const HOUSE_LABEL_DURATION: f64 = 4.5;
const HOUSE_LABEL_FADE: f64 = 0.7;
const HOUSE_LABEL_TEXT: &str = "This looks like your house. It isn't.";

/// Aviso que aparece la primera vez que Erica entra en el campo de visión del
/// jugador — funciona como tutorial diegético de la mecánica "se congela si
/// la ves" (ver entities::enemy) sin necesitar una pantalla de instrucciones.
const ERICA_LABEL_DURATION: f64 = 4.5;
const ERICA_LABEL_FADE: f64 = 0.7;
const ERICA_LABEL_TEXT: &str = "She's a little shy. Erica won't hurt you while you're looking at her.";
/// Separación vertical del aviso de Erica si llegara a coincidir con el de
/// bienvenida o el de la casa.
const ERICA_LABEL_Y_OFFSET: i32 = 34;

enum Screen {
    Welcome,
    Intro,
    Playing,
    GameOver,
    Success,
}

/// Estado del gameplay en sí — se reconstruye desde cero cada vez que se entra
/// a Playing (al terminar la intro, o al reintentar), para que cada partida
/// arranque limpia.
struct RunState {
    player: Player,
    enemy: Enemy,
    difficulty: Difficulty,
    survival_elapsed: f64,
    start_label_timer: f64,
    house_label_timer: f64,
    erica_seen: bool,
    erica_label_timer: f64,
}

impl RunState {
    fn new(difficulty: Difficulty) -> Self {
        let (spawn_row, spawn_col) = PLAYER_SPAWN_CELL;
        let (spawn_x, spawn_y) = Map::spawn_world(spawn_row, spawn_col);
        let (enemy_row, enemy_col) = ENEMY_SPAWN_CELL;
        let (enemy_x, enemy_y) = Map::spawn_world(enemy_row, enemy_col);
        RunState {
            // Ángulo PI/2 = mirando al sur, hacia una pared interior del
            // recibidor (no el pasillo abierto al este) — el jugador arranca
            // de cara a una pared, no viendo directo por un corredor.
            player: Player::new(spawn_x, spawn_y, std::f64::consts::FRAC_PI_2),
            enemy: Enemy::new(enemy_x, enemy_y),
            difficulty,
            survival_elapsed: 0.0,
            start_label_timer: START_LABEL_DURATION,
            // Arranca por encima de su propia duración: mientras el timer esté
            // por sobre HOUSE_LABEL_DURATION todavía no es su turno de
            // mostrarse (ver draw_fading_label) — así se encadena justo
            // después de que el primer aviso termina de desvanecerse.
            house_label_timer: HOUSE_LABEL_DELAY + HOUSE_LABEL_DURATION,
            erica_seen: false,
            erica_label_timer: 0.0,
        }
    }
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::PNG).map_err(|e| e.to_string())?;
    let _audio_subsystem = sdl_context.audio()?;
    let _audio_context = audio::mixer::init_audio()?;
    let mut audio_engine = AudioEngine::new()?;

    let window = video_subsystem
        .window("Not your house", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().accelerated().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    let hud_font = ttf_context
        .load_font("assets/fonts/VT323-Regular.ttf", 28)
        .map_err(|e| e.to_string())?;
    // Título: "Instrument Serif" — elegida por Cami.
    let title_font = ttf_context
        .load_font("assets/fonts/InstrumentSerif-Regular.ttf", 150)
        .map_err(|e| e.to_string())?;
    let body_font = ttf_context
        .load_font("assets/fonts/SpecialElite-Regular.ttf", 22)
        .map_err(|e| e.to_string())?;
    // UI de la bienvenida (prompt, selector de dificultad) — de vuelta a
    // Lora (Caveat no convenció una vez que las dos etapas comparten el
    // mismo wallpaper, sin la nota/XOXO que justificaba lo manuscrito).
    let ui_font = ttf_context
        .load_font("assets/fonts/Lora-SemiBold.ttf", 24)
        .map_err(|e| e.to_string())?;

    // Fondo de la bienvenida — wallpaper floral de Cami, mismo dibujo para las
    // dos etapas (título y dificultad; los intentos anteriores para la etapa
    // de dificultad no convencieron y se descartaron).
    use sdl2::image::LoadTexture;
    let title_background = texture_creator
        .load_texture("assets/ui/title_background.png")
        .map_err(|e| e.to_string())?;

    // Fondo de la cinemática de introducción — dibujo de Cami, figura oscura
    // incluida en la ilustración.
    let intro_backdrop = texture_creator
        .load_texture("assets/ui/intro_backdrop.png")
        .map_err(|e| e.to_string())?;

    // Fondo de la pantalla de éxito — dibujo de Cami.
    let success_background = texture_creator
        .load_texture("assets/ui/success_background.png")
        .map_err(|e| e.to_string())?;

    // Close-up de Erica para game over — dibujo de Cami, pantalla completa.
    let jumpscare_face = texture_creator
        .load_texture("assets/ui/jumpscare_face.png")
        .map_err(|e| e.to_string())?;

    // Buffer + textura reutilizados cada frame para el floor-casting por píxel
    // (ver render::walls::render_floor) — evita reasignar memoria cada frame.
    let floor_h = SCREEN_HEIGHT / 2;
    let mut floor_texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, SCREEN_WIDTH, floor_h)
        .map_err(|e| e.to_string())?;
    let mut floor_buffer = vec![0u8; (SCREEN_WIDTH * floor_h * 3) as usize];

    // Textura de pared — placeholder compartido por Recibidor/Dormitorio/Sala
    // hasta que Cami entregue las 3 texturas finales, una por zona (ver
    // SKILL.md, "Texturas de pared"). La Cocina no usa textura: color plano
    // generado por código.
    let mut wall_texture = texture_creator
        .load_texture("assets/textures/wallpaper.png")
        .map_err(|e| e.to_string())?;

    // Textura de piso — leída a memoria de CPU (no un Texture de GPU) para
    // poder muestrearla píxel a píxel dentro del loop de floor-casting.
    let floor_tex = render::walls::PixelTexture::load("assets/textures/floor.png")?;

    // Z-buffer reutilizado cada frame: distancia perpendicular de pared por
    // columna, lo llena render::walls y lo lee render::sprites para ocluir al
    // enemigo correctamente detrás de paredes más cercanas.
    let mut z_buffer = vec![f64::INFINITY; SCREEN_WIDTH as usize];

    sdl_context.mouse().set_relative_mouse_mode(true);

    let mut screen = Screen::Welcome;
    let mut welcome = WelcomeState::new();
    let mut intro = IntroState::new();
    let mut game_over = GameOverState::new();
    let mut success = SuccessState::new();
    let mut run = RunState::new(Difficulty::Medium);

    let mut event_pump = sdl_context.event_pump()?;
    let mut fps_counter = FpsCounter::new();
    let mut last_frame = Instant::now();

    'running: loop {
        let now = Instant::now();
        let dt = ((now - last_frame).as_secs_f64()).min(0.1);
        last_frame = now;
        fps_counter.push_frame_time(dt);

        let mut keydowns: Vec<Scancode> = Vec::new();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { scancode: Some(sc), repeat: false, .. } => {
                    if sc == Scancode::Escape {
                        break 'running;
                    }
                    keydowns.push(sc);
                }
                Event::MouseMotion { xrel, .. } => {
                    if matches!(screen, Screen::Playing) {
                        engine::camera::apply_mouse(&mut run.player, xrel);
                    }
                }
                _ => {}
            }
        }

        // === UPDATE ===
        match screen {
            Screen::Welcome => {
                welcome.update(dt);
                if welcome.handle_keydowns(&keydowns) {
                    run = RunState::new(welcome.difficulty);
                    intro = IntroState::new();
                    screen = Screen::Intro;
                }
            }
            Screen::Intro => {
                if intro.update(dt) {
                    audio_engine.start_run();
                    screen = Screen::Playing;
                }
            }
            Screen::Playing => {
                update_playing(&mut run, &event_pump, &mut audio_engine, dt);
            }
            Screen::GameOver => {
                game_over.update(dt);
                if keydowns.contains(&Scancode::R) {
                    screen = Screen::Welcome;
                }
            }
            Screen::Success => {
                success.update(dt);
                if keydowns.contains(&Scancode::Return) || keydowns.contains(&Scancode::R) {
                    screen = Screen::Welcome;
                }
            }
        }

        // === DRAW ===
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        match screen {
            Screen::Welcome => {
                welcome.draw(&mut canvas, &texture_creator, &title_font, &ui_font, &title_background, &title_background, SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32);
            }
            Screen::Intro => {
                intro.draw(&mut canvas, &texture_creator, &body_font, &intro_backdrop, SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32);
            }
            Screen::Playing => {
                draw_playing(
                    &mut canvas,
                    &texture_creator,
                    &hud_font,
                    &body_font,
                    &mut floor_texture,
                    &mut floor_buffer,
                    &floor_tex,
                    &mut wall_texture,
                    &mut z_buffer,
                    &run,
                    &fps_counter,
                );
                if run.player.is_dead() {
                    game_over = GameOverState::new();
                    audio_engine.stop_run();
                    screen = Screen::GameOver;
                } else if run.survival_elapsed >= run.difficulty.survival_seconds() {
                    success = SuccessState::new();
                    audio_engine.stop_run();
                    screen = Screen::Success;
                }
            }
            Screen::GameOver => {
                game_over.draw(&mut canvas, &texture_creator, &title_font, &body_font, &jumpscare_face, SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32);
            }
            Screen::Success => {
                success.draw(&mut canvas, &texture_creator, &title_font, &body_font, &success_background, SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32);
            }
        }

        canvas.present();
    }

    Ok(())
}

fn update_playing(run: &mut RunState, event_pump: &sdl2::EventPump, audio_engine: &mut AudioEngine, dt: f64) {
    if run.player.is_dead() {
        return;
    }

    let keyboard_state = event_pump.keyboard_state();
    engine::camera::apply_keyboard(&mut run.player, &keyboard_state, dt);
    run.enemy.update(dt, &run.player, run.difficulty.pathfind_recalc_frames());

    // Daño por contacto: mientras el enemigo esté en rango, la vida baja de
    // forma continua (no es game over instantáneo) — omnidireccional a
    // propósito, ver SKILL.md.
    let distance_to_enemy = run.enemy.distance_to_player(&run.player);
    let in_contact = distance_to_enemy < CONTACT_RANGE;
    if in_contact {
        run.player.apply_damage(DAMAGE_PER_SECOND * dt);
    }
    run.player.update_danger_flash(dt, in_contact);

    let moving = keyboard_state.is_scancode_pressed(Scancode::W) || keyboard_state.is_scancode_pressed(Scancode::S);
    audio_engine.update(dt, moving, distance_to_enemy, in_contact);

    // Primera vez que Erica entra en el campo de visión del jugador: dispara
    // el aviso tutorial (ver ERICA_LABEL_TEXT). Reutiliza el mismo chequeo de
    // FOV + línea de vista que ya existe para la IA (se congela si la ven).
    if !run.erica_seen {
        let visible = engine::camera::is_within_fov(&run.player, run.enemy.x, run.enemy.y)
            && engine::raycasting::has_line_of_sight(run.player.x, run.player.y, run.enemy.x, run.enemy.y);
        if visible {
            run.erica_seen = true;
            run.erica_label_timer = ERICA_LABEL_DURATION;
        }
    }

    run.survival_elapsed += dt;
    run.start_label_timer -= dt;
    run.house_label_timer -= dt;
    run.erica_label_timer -= dt;
}

#[allow(clippy::too_many_arguments)]
fn draw_playing(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    hud_font: &sdl2::ttf::Font,
    body_font: &sdl2::ttf::Font,
    floor_texture: &mut Texture,
    floor_buffer: &mut [u8],
    floor_tex: &render::walls::PixelTexture,
    wall_texture: &mut Texture,
    z_buffer: &mut [f64],
    run: &RunState,
    fps_counter: &FpsCounter,
) {
    render::walls::render_scene(canvas, floor_texture, floor_buffer, floor_tex, wall_texture, z_buffer, &run.player, SCREEN_WIDTH, SCREEN_HEIGHT);
    render::sprites::render_enemy(
        canvas,
        &run.player,
        &run.enemy,
        Enemy::state(&run.player),
        run.enemy.anim_flip(),
        z_buffer,
        SCREEN_WIDTH as i32,
        SCREEN_HEIGHT as i32,
    );

    // Overlay rojo vino: aviso de peligro insistente mientras el enemigo hiere
    // al jugador (ver Player::danger_flash).
    if run.player.danger_flash > 0.0 {
        let pulse = 0.75 + 0.25 * (run.survival_elapsed * 9.0).sin();
        let alpha = (run.player.danger_flash * 150.0 * pulse).clamp(0.0, 255.0) as u8;
        render::overlay::fill_screen(canvas, Color::RGB(110, 8, 26), alpha, SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32);
    }

    render::hud::draw_minimap(canvas, &run.player, SCREEN_WIDTH as i32);
    fps_counter.draw(canvas, texture_creator, hud_font);
    render::hud::draw_hearts(canvas, &run.player, 10, 40);

    // "Help is coming" y "This looks like your house..." están secuenciados
    // para no coincidir nunca entre sí (ver HOUSE_LABEL_DELAY). Si el de Erica
    // llegara a coincidir con cualquiera de los dos, se dibuja un poco más
    // arriba para no solaparse.
    let start_active = run.start_label_timer > 0.0 && run.start_label_timer <= START_LABEL_DURATION;
    let house_active = run.house_label_timer > 0.0 && run.house_label_timer <= HOUSE_LABEL_DURATION;
    let erica_y = if start_active || house_active {
        SCREEN_HEIGHT as i32 - 40 - ERICA_LABEL_Y_OFFSET
    } else {
        SCREEN_HEIGHT as i32 - 40
    };

    draw_fading_label(canvas, texture_creator, body_font, START_LABEL_TEXT, run.start_label_timer, START_LABEL_DURATION, START_LABEL_FADE, SCREEN_HEIGHT as i32 - 40);
    draw_fading_label(canvas, texture_creator, body_font, HOUSE_LABEL_TEXT, run.house_label_timer, HOUSE_LABEL_DURATION, HOUSE_LABEL_FADE, SCREEN_HEIGHT as i32 - 40);
    draw_fading_label(canvas, texture_creator, body_font, ERICA_LABEL_TEXT, run.erica_label_timer, ERICA_LABEL_DURATION, ERICA_LABEL_FADE, erica_y);
}

/// Aviso pasajero centrado: aparece, se mantiene, se desvanece — usado tanto
/// para "Help is coming" (al entrar al gameplay) como para el aviso tutorial
/// de Erica (ver SKILL.md, "Al entrar al gameplay").
#[allow(clippy::too_many_arguments)]
fn draw_fading_label(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    font: &sdl2::ttf::Font,
    text: &str,
    timer: f64,
    duration: f64,
    fade: f64,
    y: i32,
) {
    if timer <= 0.0 || timer > duration {
        return;
    }
    let elapsed = duration - timer;
    let alpha = if elapsed < fade {
        elapsed / fade
    } else if timer < fade {
        timer / fade
    } else {
        1.0
    }
    .clamp(0.0, 1.0);

    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    let color = Color::RGBA(200, 195, 190, (alpha * 255.0) as u8);
    render::text::draw_text_centered(canvas, texture_creator, font, text, SCREEN_WIDTH as i32 / 2, y, color);
}
