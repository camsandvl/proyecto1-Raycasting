//! Enemigo: persecución estilo Wolfenstein sobre el grid de bloques (mismo grid
//! que usa el jugador para colisión — ver map::BLOCK_GRID), pathfinding BFS
//! recalculado cada N frames (no cada frame, por rendimiento), y estado visual
//! (normal/enfurecido) según el % de vida del jugador. Ver SKILL.md, sección
//! "Mecánica principal: sobrevivir al enemigo".
//!
//! Dos reglas de comportamiento (inspiradas en Weeping Angels/SCP-173 y en el
//! Nemesis asomándose por una puerta) hacen que la persecución se sienta más
//! amenazante que un simple "camina en línea recta hacia vos":
//! - **Se congela si lo estás mirando** (mismo cono de FOV + línea de vista que
//!   se renderiza en pantalla): solo avanza cuando le das la espalda.
//! - **Pausa al llegar a un cruce real** (3+ salidas): un segundo "decidiendo"
//!   antes de comprometerse a un pasillo, como si estuviera olfateando.
//! Combinadas, producen el efecto de "se asoma y se queda quieto mirándote" en
//! los cruces sin necesidad de programar esa escena a mano.

use std::collections::VecDeque;

use crate::engine::{camera, raycasting};
use crate::entities::player::Player;
use crate::map::{Map, BLOCK_SIZE};

pub const ENEMY_RADIUS: f64 = 0.25;
/// Un poco más lento que el jugador (2.6 celdas/seg) — si fuera igual o más
/// rápido sería imposible perderlo por los loops del laberinto.
pub const ENEMY_SPEED: f64 = 1.9;

/// Distancia bajo la cual el enemigo está "en contacto" y hace daño.
pub const CONTACT_RANGE: f64 = 0.55;
pub const DAMAGE_PER_SECOND: f64 = 28.0;

/// Cada cuántos frames se recalcula el campo de distancias BFS. Ver SKILL.md:
/// "recalculado cada N frames (ej. cada 15-20 frames)". Difícil puede pedir un
/// valor más bajo (ej. 10) más adelante — parámetro pensado para eso.
pub const PATHFIND_RECALC_FRAMES: u32 = 18;

/// Ciclo de 2 "frames" de animación (parpadeo leve) para que el cambio de
/// estado se lea inequívocamente como animación (ver SKILL.md).
const ANIM_INTERVAL_SECS: f64 = 0.35;

/// Cuánto se queda quieto "decidiendo" al llegar a un cruce con 3+ salidas.
const JUNCTION_PAUSE_SECS: f64 = 1.2;

const UNREACHABLE: u32 = u32::MAX;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnemyState {
    Normal,
    Enraged,
}

pub struct Enemy {
    pub x: f64,
    pub y: f64,
    frames_since_recalc: u32,
    distance_field: Box<[[u32; BLOCK_SIZE]; BLOCK_SIZE]>,
    anim_timer: f64,
    anim_flip: bool,
    /// Tiempo restante de la pausa "decidiendo" en un cruce (ver JUNCTION_PAUSE_SECS).
    pause_timer: f64,
    /// ¿Estaba parado en un cruce el frame pasado? Para detectar la llegada
    /// (transición) y no reiniciar la pausa cada frame mientras sigue ahí.
    was_at_junction: bool,
    /// Expuesto para HUD/depuración y para que el sprite pueda reaccionar
    /// distinto si algún día se anima "mirando fijo" vs "caminando".
    pub frozen_by_gaze: bool,
}

impl Enemy {
    pub fn new(x: f64, y: f64) -> Self {
        Enemy {
            x,
            y,
            // Fuerza un recálculo inmediato en el primer update().
            frames_since_recalc: PATHFIND_RECALC_FRAMES,
            distance_field: Box::new([[UNREACHABLE; BLOCK_SIZE]; BLOCK_SIZE]),
            anim_timer: 0.0,
            anim_flip: false,
            pause_timer: 0.0,
            was_at_junction: false,
            frozen_by_gaze: false,
        }
    }

    /// Estado visual: umbral confirmado en SKILL.md — vida > 50% → normal,
    /// vida ≤ 50% → enfurecido. Se basa en el % de vida del jugador, no en
    /// distancia.
    pub fn state(player: &Player) -> EnemyState {
        if player.life_below_half() {
            EnemyState::Enraged
        } else {
            EnemyState::Normal
        }
    }

    /// Alterna cada `ANIM_INTERVAL_SECS` — expone el "frame" actual del ciclo
    /// de 2 frames de animación (ver render::sprites).
    pub fn anim_flip(&self) -> bool {
        self.anim_flip
    }

    pub fn distance_to_player(&self, player: &Player) -> f64 {
        let dx = self.x - player.x;
        let dy = self.y - player.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn update(&mut self, dt: f64, player: &Player, recalc_interval_frames: u32) {
        self.anim_timer += dt;
        if self.anim_timer >= ANIM_INTERVAL_SECS {
            self.anim_timer -= ANIM_INTERVAL_SECS;
            self.anim_flip = !self.anim_flip;
        }

        self.frames_since_recalc += 1;
        if self.frames_since_recalc >= recalc_interval_frames {
            self.recalc_path(player);
            self.frames_since_recalc = 0;
        }

        // Regla 1: si el jugador lo puede ver ahora mismo (mismo cono de FOV +
        // línea de vista que se renderiza en pantalla), se queda inmóvil —
        // mirar hacia otro lado es lo único que le permite seguir avanzando.
        self.frozen_by_gaze = camera::is_within_fov(player, self.x, self.y)
            && raycasting::has_line_of_sight(player.x, player.y, self.x, self.y);
        if self.frozen_by_gaze {
            return;
        }

        // Regla 2: pausa breve al llegar (recién) a un cruce real (3+ salidas)
        // — como si estuviera decidiendo hacia dónde ir.
        let row = (self.y as usize).min(BLOCK_SIZE - 1);
        let col = (self.x as usize).min(BLOCK_SIZE - 1);
        let at_junction = is_junction(row, col);
        if at_junction && !self.was_at_junction {
            self.pause_timer = JUNCTION_PAUSE_SECS;
        }
        self.was_at_junction = at_junction;

        if self.pause_timer > 0.0 {
            self.pause_timer -= dt;
            return;
        }

        self.step_towards_player(dt);
    }

    /// BFS desde la celda del jugador: genera un campo de distancias sobre
    /// todo el grid de bloques alcanzable. El enemigo luego solo necesita
    /// mirar sus 4 vecinos y bajar por el gradiente — no hace falta A* ni
    /// heurísticas para un ray caster universitario (ver SKILL.md).
    fn recalc_path(&mut self, player: &Player) {
        let target_row = (player.y as usize).min(BLOCK_SIZE - 1);
        let target_col = (player.x as usize).min(BLOCK_SIZE - 1);

        let mut field = Box::new([[UNREACHABLE; BLOCK_SIZE]; BLOCK_SIZE]);

        if Map::is_solid_idx(target_row as isize, target_col as isize) {
            // No debería pasar (el jugador nunca debería estar dentro de un
            // bloque sólido), pero por seguridad no dejamos un BFS a medio
            // inicializar si pasara.
            self.distance_field = field;
            return;
        }

        field[target_row][target_col] = 0;
        let mut queue = VecDeque::with_capacity(BLOCK_SIZE * BLOCK_SIZE);
        queue.push_back((target_row, target_col));

        while let Some((r, c)) = queue.pop_front() {
            let d = field[r][c];
            for (nr, nc) in neighbors(r, c) {
                if Map::is_solid_idx(nr as isize, nc as isize) {
                    continue;
                }
                if field[nr][nc] != UNREACHABLE {
                    continue;
                }
                field[nr][nc] = d + 1;
                queue.push_back((nr, nc));
            }
        }

        self.distance_field = field;
    }

    /// Se mueve hacia la celda vecina abierta con menor distancia del campo
    /// BFS (el "cuesta abajo" hacia el jugador).
    fn step_towards_player(&mut self, dt: f64) {
        let row = (self.y as usize).min(BLOCK_SIZE - 1);
        let col = (self.x as usize).min(BLOCK_SIZE - 1);

        let mut best: Option<(usize, usize, u32)> = None;
        for (nr, nc) in neighbors(row, col) {
            let d = self.distance_field[nr][nc];
            if d == UNREACHABLE {
                continue;
            }
            if best.is_none_or(|(_, _, bd)| d < bd) {
                best = Some((nr, nc, d));
            }
        }

        let Some((target_row, target_col, _)) = best else { return };

        let target_x = target_col as f64 + 0.5;
        let target_y = target_row as f64 + 0.5;
        let dx = target_x - self.x;
        let dy = target_y - self.y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < 1e-4 {
            return;
        }

        let step = (ENEMY_SPEED * dt).min(dist);
        self.try_move(dx / dist * step, dy / dist * step);
    }

    /// Igual que `Player::try_move`: cada eje se prueba por separado contra el
    /// grid de bloques sólidos, así el enemigo también se desliza a lo largo
    /// de una pared en vez de trabarse en las esquinas.
    fn try_move(&mut self, dx: f64, dy: f64) {
        let radius = ENEMY_RADIUS;

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

/// Vecinos ortogonales (N/S/E/W) dentro del grid, ya filtrados por límites.
fn neighbors(row: usize, col: usize) -> impl Iterator<Item = (usize, usize)> {
    let candidates =
        [(row.wrapping_sub(1), col), (row + 1, col), (row, col.wrapping_sub(1)), (row, col + 1)];
    candidates.into_iter().filter(|&(r, c)| r < BLOCK_SIZE && c < BLOCK_SIZE)
}

/// ¿Es (row, col) un cruce real (3 o más salidas abiertas)? Una esquina simple
/// (2 salidas en ángulo) no cuenta — ahí solo hay un camino posible, no hay
/// nada que "decidir".
fn is_junction(row: usize, col: usize) -> bool {
    neighbors(row, col).filter(|&(r, c)| !Map::is_solid_idx(r as isize, c as isize)).count() >= 3
}
