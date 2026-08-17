//! Definición del laberinto 10x10 (ver SKILL.md, sección "Diseño del laberinto").
//!
//! Maze DFS-perfecto con ~14% de paredes internas removidas para crear loops de
//! evasión. Generado a partir del arte ASCII del SKILL.md y validado por script
//! (conectividad 100%, 0 inconsistencias N/S/E/W entre celdas vecinas).
//!
//! El diseño se define arriba como "paredes en los bordes de la celda" (10x10
//! celdas, cada pared es un segmento sin grosor) porque así es como se dibujó y
//! validó el laberinto. Pero para el render usamos bloques sólidos estilo
//! Wolfenstein (paredes con volumen real, no líneas) — así que ese diseño se
//! convierte una sola vez, al arrancar, a un grid de doble resolución
//! (`BLOCK_GRID`, 21x21) donde cada celda original se vuelve un bloque abierto
//! de 1x1 y cada pared un bloque sólido de 1x1. Misma topología/loops, solo
//! cambia la representación para dibujar.

use std::sync::LazyLock;

pub const GRID_SIZE: usize = 10;
/// Tamaño del grid de bloques: 2*GRID_SIZE+1 (una fila/columna de bloques por
/// cada celda, más una de por medio para cada posible pared/poste).
pub const BLOCK_SIZE: usize = GRID_SIZE * 2 + 1;

/// Paredes presentes en cada uno de los 4 lados de una celda (formato "fuente
/// de verdad", tal como se dibujó el laberinto — ver conversión a bloques abajo).
#[derive(Clone, Copy, Debug)]
pub struct Walls {
    pub n: bool,
    pub s: bool,
    pub e: bool,
    pub w: bool,
}

/// Zona del apartamento (ver SKILL.md, sección "Ambientación y arte"). Ya no
/// hay una textura distinta por "cuarto" — todo el apartamento es el mismo
/// wallpaper con variaciones (manchas/marcas) mezcladas al azar por bloque —
/// excepto la zona `Drip`, que es 100% una sola variación (`wallpaper_drip.png`),
/// sin mezcla.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zone {
    /// Wallpaper base + variaciones mezcladas al azar por bloque (ver
    /// `render::walls::scatter_variant_index`).
    Normal,
    /// Filas 5-9, columnas 5-9. Spawn del enemigo. Solo `wallpaper_drip.png`,
    /// sin mezcla — la vieja zona "Cocina" (era la única con textura no-Cami).
    Drip,
}

/// Grid `[fila][columna]`, 0-indexado. Fila 0 = arriba, columna 0 = izquierda.
#[rustfmt::skip]
pub const MAZE: [[Walls; GRID_SIZE]; GRID_SIZE] = [
    [Walls{n:true,s:true,e:false,w:true}, Walls{n:true,s:false,e:false,w:false}, Walls{n:true,s:true,e:false,w:false}, Walls{n:true,s:true,e:false,w:false}, Walls{n:true,s:false,e:false,w:false}, Walls{n:true,s:true,e:false,w:false}, Walls{n:true,s:false,e:false,w:false}, Walls{n:true,s:true,e:false,w:false}, Walls{n:true,s:false,e:false,w:false}, Walls{n:true,s:false,e:true,w:false}],
    [Walls{n:true,s:false,e:true,w:true}, Walls{n:false,s:false,e:true,w:true}, Walls{n:true,s:false,e:false,w:true}, Walls{n:true,s:false,e:true,w:false}, Walls{n:false,s:false,e:false,w:true}, Walls{n:true,s:false,e:true,w:false}, Walls{n:false,s:false,e:false,w:true}, Walls{n:true,s:false,e:true,w:false}, Walls{n:false,s:false,e:false,w:true}, Walls{n:false,s:true,e:true,w:false}],
    [Walls{n:false,s:false,e:true,w:true}, Walls{n:false,s:true,e:false,w:true}, Walls{n:false,s:true,e:true,w:false}, Walls{n:false,s:true,e:false,w:true}, Walls{n:false,s:false,e:true,w:false}, Walls{n:false,s:false,e:true,w:true}, Walls{n:false,s:false,e:false,w:true}, Walls{n:false,s:false,e:true,w:false}, Walls{n:false,s:true,e:false,w:true}, Walls{n:true,s:true,e:true,w:false}],
    [Walls{n:false,s:false,e:false,w:true}, Walls{n:true,s:true,e:false,w:false}, Walls{n:true,s:false,e:false,w:false}, Walls{n:true,s:true,e:true,w:false}, Walls{n:false,s:false,e:true,w:true}, Walls{n:false,s:false,e:false,w:true}, Walls{n:false,s:true,e:true,w:false}, Walls{n:false,s:false,e:false,w:true}, Walls{n:true,s:true,e:false,w:false}, Walls{n:true,s:false,e:true,w:false}],
    [Walls{n:false,s:true,e:false,w:true}, Walls{n:true,s:true,e:true,w:false}, Walls{n:false,s:true,e:false,w:true}, Walls{n:true,s:false,e:true,w:false}, Walls{n:false,s:true,e:false,w:true}, Walls{n:false,s:true,e:false,w:false}, Walls{n:true,s:true,e:false,w:false}, Walls{n:false,s:false,e:true,w:false}, Walls{n:true,s:false,e:false,w:true}, Walls{n:false,s:false,e:true,w:false}],
    [Walls{n:true,s:false,e:false,w:true}, Walls{n:true,s:true,e:false,w:false}, Walls{n:true,s:true,e:false,w:false}, Walls{n:false,s:false,e:true,w:false}, Walls{n:true,s:false,e:false,w:true}, Walls{n:true,s:true,e:false,w:false}, Walls{n:true,s:false,e:true,w:false}, Walls{n:false,s:false,e:true,w:true}, Walls{n:false,s:false,e:true,w:true}, Walls{n:false,s:false,e:true,w:true}],
    [Walls{n:false,s:false,e:false,w:true}, Walls{n:true,s:false,e:true,w:false}, Walls{n:true,s:true,e:false,w:true}, Walls{n:false,s:false,e:false,w:false}, Walls{n:false,s:true,e:false,w:false}, Walls{n:true,s:true,e:true,w:false}, Walls{n:false,s:false,e:true,w:true}, Walls{n:false,s:false,e:true,w:true}, Walls{n:false,s:true,e:false,w:true}, Walls{n:false,s:false,e:true,w:false}],
    [Walls{n:false,s:false,e:false,w:true}, Walls{n:false,s:true,e:false,w:false}, Walls{n:true,s:false,e:true,w:false}, Walls{n:false,s:true,e:false,w:true}, Walls{n:true,s:false,e:true,w:false}, Walls{n:true,s:false,e:false,w:true}, Walls{n:false,s:false,e:true,w:false}, Walls{n:false,s:true,e:false,w:true}, Walls{n:true,s:true,e:false,w:false}, Walls{n:false,s:false,e:true,w:false}],
    [Walls{n:false,s:true,e:false,w:true}, Walls{n:true,s:false,e:true,w:false}, Walls{n:false,s:false,e:true,w:true}, Walls{n:true,s:true,e:false,w:true}, Walls{n:false,s:true,e:false,w:false}, Walls{n:false,s:true,e:true,w:false}, Walls{n:false,s:true,e:true,w:true}, Walls{n:true,s:false,e:false,w:true}, Walls{n:true,s:false,e:false,w:false}, Walls{n:false,s:true,e:true,w:false}],
    [Walls{n:true,s:true,e:false,w:true}, Walls{n:false,s:true,e:true,w:false}, Walls{n:false,s:true,e:false,w:true}, Walls{n:true,s:true,e:false,w:false}, Walls{n:true,s:true,e:false,w:false}, Walls{n:true,s:true,e:false,w:false}, Walls{n:true,s:true,e:false,w:false}, Walls{n:false,s:true,e:false,w:false}, Walls{n:false,s:true,e:false,w:false}, Walls{n:true,s:true,e:true,w:false}],
];

/// Celda de spawn del jugador (esquina superior izquierda), en coordenadas de
/// celda ORIGINAL (0..GRID_SIZE) — no de bloque. Usar `Map::spawn_world` para
/// convertir a coordenadas de mundo.
pub const PLAYER_SPAWN_CELL: (usize, usize) = (0, 0); // (row, col)
/// Celda de spawn del enemigo (esquina inferior derecha).
pub const ENEMY_SPAWN_CELL: (usize, usize) = (9, 9); // (row, col)

/// Devuelve la zona de una celda ORIGINAL dada (fila, columna).
pub fn zone_of(row: usize, col: usize) -> Zone {
    if row >= 5 && col >= 5 {
        Zone::Drip
    } else {
        Zone::Normal
    }
}

/// Construye el grid de bloques (21x21) a partir de `MAZE`. Ver comentario del
/// módulo. `BLOCK_GRID[fila][columna] == true` significa "bloque sólido".
fn build_block_grid() -> [[bool; BLOCK_SIZE]; BLOCK_SIZE] {
    let mut blocks = [[false; BLOCK_SIZE]; BLOCK_SIZE];

    // Postes en cada intersección del grid (columnas de soporte, como en un
    // apartamento real): siempre sólidos. No afectan la conectividad porque el
    // movimiento nunca corta en diagonal por una esquina, solo por los
    // segmentos N/S/E/W de abajo.
    for i in 0..=GRID_SIZE {
        for j in 0..=GRID_SIZE {
            blocks[2 * i][2 * j] = true;
        }
    }

    // Segmentos verticales: pared E/W entre celdas horizontalmente adyacentes
    // (más los bordes oeste/este del mapa).
    for i in 0..GRID_SIZE {
        blocks[2 * i + 1][0] = MAZE[i][0].w;
        for j in 0..GRID_SIZE {
            if j + 1 < GRID_SIZE {
                blocks[2 * i + 1][2 * j + 2] = MAZE[i][j].e;
            }
        }
        blocks[2 * i + 1][2 * GRID_SIZE] = MAZE[i][GRID_SIZE - 1].e;
    }

    // Segmentos horizontales: pared N/S entre celdas verticalmente adyacentes
    // (más los bordes norte/sur del mapa). Las celdas interiores (índices
    // impar/impar) quedan en `false` (abiertas) por el valor inicial del array.
    for j in 0..GRID_SIZE {
        blocks[0][2 * j + 1] = MAZE[0][j].n;
        for i in 0..GRID_SIZE {
            if i + 1 < GRID_SIZE {
                blocks[2 * i + 2][2 * j + 1] = MAZE[i][j].s;
            }
        }
        blocks[2 * GRID_SIZE][2 * j + 1] = MAZE[GRID_SIZE - 1][j].s;
    }

    blocks
}

pub static BLOCK_GRID: LazyLock<[[bool; BLOCK_SIZE]; BLOCK_SIZE]> = LazyLock::new(build_block_grid);

/// Convierte un índice de bloque (0..BLOCK_SIZE) a su celda original más
/// cercana (0..GRID_SIZE) — exacto para índices impares (interior de celda),
/// aproximado para índices pares (pared/poste, redondea a la celda vecina).
/// Suficiente para elegir textura de zona; no necesita ser exacto en el borde.
fn orig_index(block_idx: usize) -> usize {
    (block_idx.saturating_sub(1) / 2).min(GRID_SIZE - 1)
}

/// Zona temática de un bloque del grid de render (ver `zone_of` para celdas originales).
pub fn zone_of_block(block_row: usize, block_col: usize) -> Zone {
    zone_of(orig_index(block_row), orig_index(block_col))
}

pub struct Map;

impl Map {
    /// ¿Está el bloque (row, col) dentro del grid de bloques?
    pub fn block_in_bounds(row: isize, col: isize) -> bool {
        row >= 0 && col >= 0 && (row as usize) < BLOCK_SIZE && (col as usize) < BLOCK_SIZE
    }

    /// ¿Es sólido el bloque en el índice (row, col)? Fuera de rango cuenta como
    /// sólido (evita que el DDA se salga del grid).
    pub fn is_solid_idx(row: isize, col: isize) -> bool {
        if !Self::block_in_bounds(row, col) {
            return true;
        }
        BLOCK_GRID[row as usize][col as usize]
    }

    /// ¿Es sólido el bloque que contiene el punto de mundo (x, y)?
    pub fn is_solid_block(x: f64, y: f64) -> bool {
        if x < 0.0 || y < 0.0 {
            return true;
        }
        Self::is_solid_idx(y as isize, x as isize)
    }

    /// Centro, en coordenadas de mundo (grid de bloques), de la celda ORIGINAL
    /// (row, col). Usado para posicionar el spawn del jugador/enemigo.
    pub fn spawn_world(orig_row: usize, orig_col: usize) -> (f64, f64) {
        (2.0 * orig_col as f64 + 1.5, 2.0 * orig_row as f64 + 1.5)
    }
}
