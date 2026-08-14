//! Geometría del minimapa (sin dependencias de SDL): dónde va cada celda en
//! píxeles, dónde cae el marcador del jugador. El dibujo real (líneas, rects)
//! vive en `render::hud`, que consume este layout.

use crate::entities::player::Player;
use crate::map::BLOCK_SIZE;

pub const MARGIN: i32 = 12;
/// Grid de bloques es 21x21 (el doble de GRID_SIZE) — celda más chica en
/// píxeles para que el minimapa mantenga un tamaño total similar al de antes.
pub const CELL_PX: i32 = 4;

pub struct MinimapLayout {
    /// Esquina superior izquierda del área del minimapa, en píxeles de pantalla.
    pub origin_x: i32,
    pub origin_y: i32,
    pub cell_px: i32,
    pub size_px: i32,
}

/// Minimapa anclado en la esquina superior derecha (spec: "en una esquina, no
/// lado a lado").
pub fn layout(screen_w: i32) -> MinimapLayout {
    let size_px = CELL_PX * BLOCK_SIZE as i32;
    MinimapLayout { origin_x: screen_w - size_px - MARGIN, origin_y: MARGIN, cell_px: CELL_PX, size_px }
}

/// Posición del jugador en píxeles de pantalla dentro del área del minimapa.
pub fn world_to_minimap_px(layout: &MinimapLayout, world_x: f64, world_y: f64) -> (i32, i32) {
    let px = layout.origin_x + (world_x * layout.cell_px as f64) as i32;
    let py = layout.origin_y + (world_y * layout.cell_px as f64) as i32;
    (px, py)
}

pub fn player_marker_px(layout: &MinimapLayout, player: &Player) -> (i32, i32) {
    world_to_minimap_px(layout, player.x, player.y)
}
