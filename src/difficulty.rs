//! Dificultad seleccionable (ver SKILL.md, "Dificultad seleccionable"): 3
//! tiempos de supervivencia sobre el mismo mapa/assets. En difícil además se
//! acelera un poco el recálculo del pathfinding del enemigo (recomendación
//! opcional del SKILL.md) para que se sienta realmente más difícil, no solo
//! más largo.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub fn survival_seconds(self) -> f64 {
        match self {
            Difficulty::Easy => 90.0,
            Difficulty::Medium => 150.0,
            Difficulty::Hard => 210.0,
        }
    }

    /// Cada cuántos frames se recalcula el BFS del enemigo — ver
    /// `entities::enemy::PATHFIND_RECALC_FRAMES` para el valor base.
    pub fn pathfind_recalc_frames(self) -> u32 {
        match self {
            Difficulty::Easy => 16,
            Difficulty::Medium => 12,
            Difficulty::Hard => 6,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Difficulty::Easy => "EASY",
            Difficulty::Medium => "MEDIUM",
            Difficulty::Hard => "HARD",
        }
    }
}
