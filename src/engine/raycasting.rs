//! Algoritmo DDA (Digital Differential Analyzer) para raycasting estilo
//! Wolfenstein — grid de bloques sólidos, el clásico de verdad: cada celda del
//! grid de render es pared (sólida) o piso (abierta), sin casos especiales por
//! borde. Ver `map::BLOCK_GRID` para cómo se construye ese grid a partir del
//! diseño del laberinto.

use crate::map::{zone_of_block, Map, Zone};

/// Orientación de la cara de pared golpeada por el rayo. Se usa para diferenciar
/// sombreado (paredes E/W un poco más oscuras que N/S, como en Wolfenstein) y para
/// elegir qué coordenada usar al calcular el offset de textura.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// Golpeó una cara vertical del bloque (lado Este u Oeste).
    Vertical,
    /// Golpeó una cara horizontal del bloque (lado Norte o Sur).
    Horizontal,
}

#[derive(Clone, Copy, Debug)]
pub struct RayHit {
    /// Distancia perpendicular al plano de cámara (ya corregida de "ojo de pez").
    pub perp_dist: f64,
    /// Posición a lo largo de la cara golpeada, 0.0-1.0, para mapear textura en X.
    pub wall_x: f64,
    pub side: Side,
    pub zone: Zone,
    pub block_row: usize,
    pub block_col: usize,
}

const MAX_STEPS: i32 = 128;

/// Lanza un rayo desde (pos_x, pos_y) en la dirección (dir_x, dir_y) y devuelve el
/// primer bloque sólido golpeado, si lo hay dentro de MAX_STEPS.
pub fn cast_ray(pos_x: f64, pos_y: f64, dir_x: f64, dir_y: f64) -> Option<RayHit> {
    let mut map_x = pos_x.floor() as isize;
    let mut map_y = pos_y.floor() as isize;

    if !Map::block_in_bounds(map_y, map_x) {
        return None;
    }

    let delta_dist_x = if dir_x.abs() < 1e-12 { f64::INFINITY } else { (1.0 / dir_x).abs() };
    let delta_dist_y = if dir_y.abs() < 1e-12 { f64::INFINITY } else { (1.0 / dir_y).abs() };

    let (step_x, mut side_dist_x) = if dir_x < 0.0 {
        (-1isize, (pos_x - map_x as f64) * delta_dist_x)
    } else {
        (1isize, (map_x as f64 + 1.0 - pos_x) * delta_dist_x)
    };
    let (step_y, mut side_dist_y) = if dir_y < 0.0 {
        (-1isize, (pos_y - map_y as f64) * delta_dist_y)
    } else {
        (1isize, (map_y as f64 + 1.0 - pos_y) * delta_dist_y)
    };

    for _ in 0..MAX_STEPS {
        let side = if side_dist_x < side_dist_y {
            side_dist_x += delta_dist_x;
            map_x += step_x;
            Side::Vertical
        } else {
            side_dist_y += delta_dist_y;
            map_y += step_y;
            Side::Horizontal
        };

        if !Map::block_in_bounds(map_y, map_x) {
            return None;
        }

        if Map::is_solid_idx(map_y, map_x) {
            let perp_dist = if side == Side::Vertical {
                side_dist_x - delta_dist_x
            } else {
                side_dist_y - delta_dist_y
            }
            .max(1e-6);

            let wall_x = if side == Side::Vertical {
                let wy = pos_y + perp_dist * dir_y;
                wy - wy.floor()
            } else {
                let wx = pos_x + perp_dist * dir_x;
                wx - wx.floor()
            };

            let row = map_y as usize;
            let col = map_x as usize;
            return Some(RayHit {
                perp_dist,
                wall_x,
                side,
                zone: zone_of_block(row, col),
                block_row: row,
                block_col: col,
            });
        }
    }

    None
}

/// ¿Hay línea de vista despejada entre dos puntos (sin pared sólida en medio)?
/// Usado para exigir que el jugador realmente pueda ver al enemigo antes de
/// que este pueda herirlo — ver `engine::camera::is_within_fov` y su uso en
/// main.rs: la persecución debe sentirse inminente porque se ve venir, no
/// como una emboscada invisible por detrás de la cámara.
pub fn has_line_of_sight(from_x: f64, from_y: f64, to_x: f64, to_y: f64) -> bool {
    let dx = to_x - from_x;
    let dy = to_y - from_y;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist < 1e-6 {
        return true;
    }
    match cast_ray(from_x, from_y, dx / dist, dy / dist) {
        // Margen chico: no contar el propio bloque en el que está parado el
        // objetivo como si fuera una pared tapándolo.
        Some(hit) => hit.perp_dist >= dist - 0.2,
        None => true,
    }
}
