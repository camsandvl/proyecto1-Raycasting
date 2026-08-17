//! Motor de audio: música con volumen dinámico según distancia al enemigo (el
//! "disco" del SKILL.md, 3 saltos discretos) + SFX (pasos, detección/ataque,
//! ambiente). Los archivos en `assets/audio/` son placeholders generados por
//! código (tonos simples) — Fase 5 los reemplaza por los de Cami sin tocar
//! esta lógica, solo cambian los `.wav`/`.ogg`/`.mp3` en disco.

use sdl2::mixer::{self, Channel, Chunk, InitFlag, Music, AUDIO_S16LSB, DEFAULT_CHANNELS};

// Volumen del "disco" (música) en 3 saltos discretos — ver SKILL.md, sección
// Audio. 0-128 es el rango de sdl2::mixer.
const MUSIC_VOLUME_FAR: i32 = 26; // ~20%
const MUSIC_VOLUME_MEDIUM: i32 = 70; // ~55%
const MUSIC_VOLUME_NEAR: i32 = 128; // 100%

// Umbrales de distancia — el SKILL.md los define en "celdas" del maze
// original de 10x10 (>7 lejos, 4-7 medio, <4 cerca). El jugador/enemigo viven
// en el grid de bloques de doble resolución (ver map.rs), así que se duplican
// acá para comparar en las mismas unidades que `Enemy::distance_to_player`.
const DIST_FAR_BLOCKS: f64 = 14.0;
const DIST_MEDIUM_BLOCKS: f64 = 8.0;

const FOOTSTEP_INTERVAL_SECS: f64 = 0.42;

/// Debe mantenerse viva mientras el juego use audio (cierra el subsistema al
/// dropearse) — se guarda en una variable de `main()` sin usarse directamente.
pub struct AudioContext {
    _mixer: mixer::Sdl2MixerContext,
}

/// Abre el dispositivo de audio e inicializa `sdl2::mixer`. Llamar una sola
/// vez, después de `sdl_context.audio()`.
pub fn init_audio() -> Result<AudioContext, String> {
    let frequency = 44_100;
    let format = AUDIO_S16LSB;
    let channels = DEFAULT_CHANNELS;
    let chunk_size = 1024;
    mixer::open_audio(frequency, format, channels, chunk_size)?;
    let mixer_context = mixer::init(InitFlag::OGG | InitFlag::MP3)?;
    mixer::allocate_channels(8);
    Ok(AudioContext { _mixer: mixer_context })
}

pub struct AudioEngine {
    footstep: Chunk,
    detect: Chunk,
    ambient: Chunk,
    music: Music<'static>,
    step_timer: f64,
    was_in_contact: bool,
    ambient_started: bool,
}

impl AudioEngine {
    pub fn new() -> Result<Self, String> {
        let mut footstep = Chunk::from_file("assets/audio/sfx/footstep.wav")?;
        let mut detect = Chunk::from_file("assets/audio/sfx/detect.wav")?;
        let ambient = Chunk::from_file("assets/audio/sfx/ambient.wav")?;
        let music = Music::from_file("assets/audio/music/theme.wav")?;
        footstep.set_volume(90);
        detect.set_volume(110);
        Ok(AudioEngine {
            footstep,
            detect,
            ambient,
            music,
            step_timer: 0.0,
            was_in_contact: false,
            ambient_started: false,
        })
    }

    /// Llamar al arrancar una partida (transición Intro → Playing): música +
    /// ambiente en loop, timers en cero.
    pub fn start_run(&mut self) {
        Music::set_volume(MUSIC_VOLUME_FAR);
        let _ = self.music.play(-1);
        if !self.ambient_started && Channel::all().play(&self.ambient, -1).is_ok() {
            self.ambient_started = true;
        }
        self.step_timer = 0.0;
        self.was_in_contact = false;
    }

    /// Llamar al salir de Playing (game over o éxito): corta todo audio de la
    /// partida en curso.
    pub fn stop_run(&mut self) {
        Music::halt();
        Channel::all().halt();
        self.ambient_started = false;
    }

    /// Actualiza volumen dinámico + dispara SFX. `enemy_distance_blocks` en
    /// las mismas unidades que `Enemy::distance_to_player`.
    pub fn update(&mut self, dt: f64, moving: bool, enemy_distance_blocks: f64, in_contact: bool) {
        let volume = if enemy_distance_blocks > DIST_FAR_BLOCKS {
            MUSIC_VOLUME_FAR
        } else if enemy_distance_blocks > DIST_MEDIUM_BLOCKS {
            MUSIC_VOLUME_MEDIUM
        } else {
            MUSIC_VOLUME_NEAR
        };
        Music::set_volume(volume);

        if moving {
            self.step_timer -= dt;
            if self.step_timer <= 0.0 {
                self.step_timer = FOOTSTEP_INTERVAL_SECS;
                let _ = Channel::all().play(&self.footstep, 0);
            }
        } else {
            self.step_timer = 0.0;
        }

        // Detección/ataque: dispara en el flanco de subida del contacto, no
        // todos los frames que dure (sería un ruido continuo insoportable).
        if in_contact && !self.was_in_contact {
            let _ = Channel::all().play(&self.detect, 0);
        }
        self.was_in_contact = in_contact;
    }
}
