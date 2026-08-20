//! Motor de audio: solo dos elementos — música de fondo en loop constante
//! (`assets/audio/music/theme.mp3`) y el SFX de pasos (`footstep.wav`). Sin
//! "disco" con volumen dinámico por distancia ni SFX de detección/ambiente —
//! se probaron y se sacaron a pedido explícito, mantener la mezcla simple.

use sdl2::mixer::{self, Channel, Chunk, InitFlag, Music, AUDIO_S16LSB, DEFAULT_CHANNELS};

/// Volumen fijo de la música — 0-128 es el rango de sdl2::mixer.
const MUSIC_VOLUME: i32 = 90;

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
    music: Music<'static>,
    step_timer: f64,
}

impl AudioEngine {
    pub fn new() -> Result<Self, String> {
        let mut footstep = Chunk::from_file("assets/audio/sfx/footstep.wav")?;
        let music = Music::from_file("assets/audio/music/theme.mp3")?;
        footstep.set_volume(90);
        Ok(AudioEngine { footstep, music, step_timer: 0.0 })
    }

    /// Llamar al arrancar una partida (transición Welcome → Intro, así la
    /// música ya está sonando durante la cinemática): música en loop, timer
    /// de pasos en cero.
    pub fn start_run(&mut self) {
        Music::set_volume(MUSIC_VOLUME);
        let _ = self.music.play(-1);
        self.step_timer = 0.0;
    }

    /// Llamar al salir de Playing (game over o éxito): corta todo audio de la
    /// partida en curso.
    pub fn stop_run(&mut self) {
        Music::halt();
        Channel::all().halt();
    }

    /// Dispara el SFX de pasos mientras el jugador se mueve, cada
    /// `FOOTSTEP_INTERVAL_SECS`.
    pub fn update(&mut self, dt: f64, moving: bool) {
        if moving {
            self.step_timer -= dt;
            if self.step_timer <= 0.0 {
                self.step_timer = FOOTSTEP_INTERVAL_SECS;
                let _ = Channel::all().play(&self.footstep, 0);
            }
        } else {
            self.step_timer = 0.0;
        }
    }
}
