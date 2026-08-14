//! Ray Caster de terror — punto de entrada. Ver SKILL.md para el diseño completo.
//!
//! Fase 1 (motor base): ventana SDL2, raycasting DDA con paredes placeholder,
//! movimiento + colisión + rotación (teclado y mouse), contador de FPS.

mod engine;
mod entities;
mod map;
mod render;

use std::time::Instant;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};

use entities::enemy::{Enemy, CONTACT_RANGE, DAMAGE_PER_SECOND, PATHFIND_RECALC_FRAMES};
use entities::player::Player;
use map::{Map, ENEMY_SPAWN_CELL, PLAYER_SPAWN_CELL};
use render::hud::FpsCounter;

const SCREEN_WIDTH: u32 = 960;
const SCREEN_HEIGHT: u32 = 600;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let window = video_subsystem
        .window("Ray Caster de Terror", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().accelerated().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    let hud_font = ttf_context
        .load_font("assets/fonts/VT323-Regular.ttf", 28)
        .map_err(|e| e.to_string())?;

    // Buffer + textura reutilizados cada frame para el floor-casting por píxel
    // (ver render::walls::render_floor) — evita reasignar memoria cada frame.
    let floor_h = SCREEN_HEIGHT / 2;
    let mut floor_texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, SCREEN_WIDTH, floor_h)
        .map_err(|e| e.to_string())?;
    let mut floor_buffer = vec![0u8; (SCREEN_WIDTH * floor_h * 3) as usize];

    sdl_context.mouse().set_relative_mouse_mode(true);

    let (spawn_row, spawn_col) = PLAYER_SPAWN_CELL;
    let (spawn_x, spawn_y) = Map::spawn_world(spawn_row, spawn_col);
    // Ángulo inicial 0.0 = mirando al este, hacia el pasillo abierto del recibidor.
    let mut player = Player::new(spawn_x, spawn_y, 0.0);

    let (enemy_row, enemy_col) = ENEMY_SPAWN_CELL;
    let (enemy_x, enemy_y) = Map::spawn_world(enemy_row, enemy_col);
    let mut enemy = Enemy::new(enemy_x, enemy_y);

    // Z-buffer reutilizado cada frame: distancia perpendicular de pared por
    // columna, lo llena render::walls y lo lee render::sprites para ocluir al
    // enemigo correctamente detrás de paredes más cercanas.
    let mut z_buffer = vec![f64::INFINITY; SCREEN_WIDTH as usize];

    let mut event_pump = sdl_context.event_pump()?;
    let mut fps_counter = FpsCounter::new();
    let mut last_frame = Instant::now();
    // Reloj propio (no system time) para el pulso del overlay de peligro.
    let mut clock = 0.0f64;

    'running: loop {
        let now = Instant::now();
        let dt = (now - last_frame).as_secs_f64();
        last_frame = now;
        clock += dt;
        fps_counter.push_frame_time(dt);

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                Event::MouseMotion { xrel, .. } => {
                    engine::camera::apply_mouse(&mut player, xrel);
                }
                _ => {}
            }
        }

        let keyboard_state = event_pump.keyboard_state();
        if !player.is_dead() {
            engine::camera::apply_keyboard(&mut player, &keyboard_state, dt);
            enemy.update(dt, &player, PATHFIND_RECALC_FRAMES);

            // Daño por contacto: mientras el enemigo esté en rango, la vida baja
            // de forma continua (no es game over instantáneo) — ver SKILL.md.
            // Omnidireccional a propósito (igual que la gran mayoría del género):
            // que te pueda alcanzar sin que estés mirando es justamente lo que
            // genera la tensión de "¿estará detrás mío?".
            let in_contact = enemy.distance_to_player(&player) < CONTACT_RANGE;
            if in_contact {
                player.apply_damage(DAMAGE_PER_SECOND * dt);
            }
            player.update_danger_flash(dt, in_contact);
        }
        // TODO Fase 6: al morir, congelar el frame y mostrar el jumpscare en vez
        // de simplemente dejar de actualizar el mundo.

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        render::walls::render_scene(
            &mut canvas,
            &mut floor_texture,
            &mut floor_buffer,
            &mut z_buffer,
            &player,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
        );
        render::sprites::render_enemy(
            &mut canvas,
            &player,
            &enemy,
            Enemy::state(&player),
            enemy.anim_flip(),
            &z_buffer,
            SCREEN_WIDTH as i32,
            SCREEN_HEIGHT as i32,
        );
        // Overlay rojo vino: aviso de peligro insistente mientras el enemigo
        // hiere al jugador (sube rápido, baja lento — ver Player::danger_flash),
        // más difícil de ignorar que solo leer el número de vida.
        if player.danger_flash > 0.0 {
            let pulse = 0.75 + 0.25 * (clock * 9.0).sin();
            let alpha = (player.danger_flash * 150.0 * pulse).clamp(0.0, 255.0) as u8;
            render::overlay::fill_screen(
                &mut canvas,
                Color::RGB(110, 8, 26),
                alpha,
                SCREEN_WIDTH as i32,
                SCREEN_HEIGHT as i32,
            );
        }

        render::hud::draw_minimap(&mut canvas, &player, SCREEN_WIDTH as i32);
        fps_counter.draw(&mut canvas, &texture_creator, &hud_font);
        render::hud::draw_life(&mut canvas, &texture_creator, &hud_font, &player);

        canvas.present();
    }

    Ok(())
}
