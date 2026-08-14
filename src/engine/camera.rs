//! Traduce input (teclado + mouse) a movimiento del jugador, y deriva los vectores
//! de dirección/plano de cámara que usa el renderer para lanzar los rayos.

use sdl2::keyboard::{KeyboardState, Scancode};

use crate::entities::player::Player;

/// Campo de visión horizontal, en radianes
pub const FOV: f64 = 66.0 * std::f64::consts::PI / 180.0;

/// Vectores de dirección y plano de cámara para un frame de render dado.
pub struct CameraView {
    pub dir_x: f64,
    pub dir_y: f64,
    pub plane_x: f64,
    pub plane_y: f64,
}

pub fn camera_view(player: &Player) -> CameraView {
    let (dir_x, dir_y) = player.dir();
    let plane_len = (FOV / 2.0).tan();
    // Perpendicular a `dir`, escalado por el FOV. Ver player.rs / raycasting.rs para
    // la convención de ejes (x = este, y = sur, ángulo 0 = mirando al este).
    let plane_x = dir_y * plane_len;
    let plane_y = -dir_x * plane_len;
    CameraView { dir_x, dir_y, plane_x, plane_y }
}

/// ¿Cae `(target_x, target_y)` dentro del mismo cono de visión que se
/// renderiza en pantalla? Usado para exigir que el enemigo esté realmente a
/// la vista antes de poder hacer daño — ver uso en main.rs.
pub fn is_within_fov(player: &Player, target_x: f64, target_y: f64) -> bool {
    let dx = target_x - player.x;
    let dy = target_y - player.y;
    let dist_sq = dx * dx + dy * dy;
    if dist_sq < 1e-9 {
        return true;
    }
    let dist = dist_sq.sqrt();
    let (dir_x, dir_y) = player.dir();
    let dot = (dx / dist) * dir_x + (dy / dist) * dir_y;
    dot >= (FOV / 2.0).cos()
}

/// Aplica el estado del teclado (W/S mover, A/D rotar) al jugador para este frame.
/// `dt` en segundos.
pub fn apply_keyboard(player: &mut Player, keys: &KeyboardState, dt: f64) {
    use crate::entities::player::{MOVE_SPEED, ROT_SPEED};

    if keys.is_scancode_pressed(Scancode::W) {
        player.move_forward(MOVE_SPEED * dt);
    }
    if keys.is_scancode_pressed(Scancode::S) {
        player.move_forward(-MOVE_SPEED * dt);
    }
    if keys.is_scancode_pressed(Scancode::A) {
        player.rotate(-ROT_SPEED * dt);
    }
    if keys.is_scancode_pressed(Scancode::D) {
        player.rotate(ROT_SPEED * dt);
    }
}

/// Aplica la rotación adicional por movimiento horizontal del mouse (rotación
/// relativa, capturada en modo "relative mouse mode" de SDL2).
pub fn apply_mouse(player: &mut Player, xrel: i32) {
    use crate::entities::player::MOUSE_SENSITIVITY;

    if xrel != 0 {
        player.rotate(xrel as f64 * MOUSE_SENSITIVITY);
    }
}
