//! Jugador: posición, ángulo de cámara, vida y movimiento con colisión.

use crate::map::Map;

/// Radio de colisión del jugador, en unidades de celda (1.0 == una celda del grid).
/// Deja margen para pasillos de 1 celda de ancho sin que la cámara se pegue a la pared.
pub const PLAYER_RADIUS: f64 = 0.2;

pub const MOVE_SPEED: f64 = 2.6; // celdas por segundo
pub const ROT_SPEED: f64 = 2.5; // radianes por segundo (teclado A/D)
pub const MOUSE_SENSITIVITY: f64 = 0.0022; // radianes por pixel relativo de mouse

pub const MAX_LIFE: f64 = 100.0;

/// Velocidad a la que sube el aviso de peligro (overlay rojo) al recibir daño
/// — rápido, para que se sienta inmediato.
const DANGER_RISE_PER_SEC: f64 = 6.0;
/// Velocidad a la que baja cuando el enemigo ya no está en contacto — lento a
/// propósito, para que el aviso se sienta "insistente" y no desaparezca en
/// cuanto das un paso atrás (el jugador debe alejarse de verdad, no solo salir
/// del rango un instante).
const DANGER_FALL_PER_SEC: f64 = 1.6;

pub struct Player {
    pub x: f64,
    pub y: f64,
    /// Ángulo de la cámara en radianes. 0 = mirando hacia +x (este).
    pub angle: f64,
    pub life: f64,
    /// 0.0..1.0 — qué tan "en peligro" está ahora mismo (el enemigo lo está
    /// hiriendo). Alimenta el overlay rojo de pantalla (render::overlay) para
    /// que quede claro que el enemigo te alcanzó y sigue encima, más allá de
    /// leer el número de vida en el HUD.
    pub danger_flash: f64,
}

impl Player {
    pub fn new(x: f64, y: f64, angle: f64) -> Self {
        Player { x, y, angle, life: MAX_LIFE, danger_flash: 0.0 }
    }

    pub fn dir(&self) -> (f64, f64) {
        (self.angle.cos(), self.angle.sin())
    }

    pub fn is_dead(&self) -> bool {
        self.life <= 0.0
    }

    fn life_fraction(&self) -> f64 {
        (self.life / MAX_LIFE).clamp(0.0, 1.0)
    }

    /// ¿Está el jugador en estado "enfurecido" para el enemigo? (delegado aquí solo
    /// como dato; la decisión de qué sprite mostrar vive en entities::enemy).
    pub fn life_below_half(&self) -> bool {
        self.life_fraction() <= 0.5
    }

    /// Resta vida (nunca baja de 0). Usado cuando el enemigo está en rango de
    /// contacto — ver `entities::enemy::CONTACT_RANGE`.
    pub fn apply_damage(&mut self, amount: f64) {
        self.life = (self.life - amount).max(0.0);
    }

    /// Actualiza el aviso de peligro (ver `danger_flash`). Llamar una vez por
    /// frame con si el enemigo está en rango de contacto ahora mismo.
    pub fn update_danger_flash(&mut self, dt: f64, in_contact: bool) {
        if in_contact {
            self.danger_flash = (self.danger_flash + DANGER_RISE_PER_SEC * dt).min(1.0);
        } else {
            self.danger_flash = (self.danger_flash - DANGER_FALL_PER_SEC * dt).max(0.0);
        }
    }

    pub fn rotate(&mut self, delta_rad: f64) {
        self.angle += delta_rad;
        // Mantener el ángulo en un rango razonable evita pérdida de precisión tras
        // muchas vueltas en partidas largas.
        let two_pi = std::f64::consts::TAU;
        if self.angle > two_pi {
            self.angle -= two_pi;
        } else if self.angle < -two_pi {
            self.angle += two_pi;
        }
    }

    /// Mueve al jugador `amount` celdas en la dirección de la cámara (negativo =
    /// retroceder), deslizando a lo largo de paredes y sin nunca atravesarlas.
    pub fn move_forward(&mut self, amount: f64) {
        let (dx_dir, dy_dir) = self.dir();
        self.try_move(dx_dir * amount, dy_dir * amount);
    }

    /// Intenta desplazar al jugador por (dx, dy). Cada eje se prueba por
    /// separado contra el grid de bloques sólidos (permite "deslizarse" a lo
    /// largo de una pared en vez de trabarse en seco al chocar en diagonal):
    /// se prueba un punto de sonda a `PLAYER_RADIUS` en la dirección del
    /// movimiento; si cae dentro de un bloque sólido, ese eje no se mueve.
    fn try_move(&mut self, dx: f64, dy: f64) {
        let radius = PLAYER_RADIUS;

        if dx != 0.0 {
            let new_x = self.x + dx;
            let probe_x = if dx > 0.0 { new_x + radius } else { new_x - radius };
            if !Map::is_solid_block(probe_x, self.y) {
                self.x = new_x;
            }
        }

        if dy != 0.0 {
            let new_y = self.y + dy;
            let probe_y = if dy > 0.0 { new_y + radius } else { new_y - radius };
            if !Map::is_solid_block(self.x, probe_y) {
                self.y = new_y;
            }
        }
    }
}
