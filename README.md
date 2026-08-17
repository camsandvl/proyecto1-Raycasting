# Not your house

Proyecto universitario (gráficas por computadora, parte 3): un ray caster estilo
Wolfenstein 3D. Es tu aniversario. Erica, tu novia, ha estado actuando raro. Esta
no es tu casa.

Ver [SKILL.md](SKILL.md) para el diseño completo (scope, mecánicas, guion narrativo,
decisiones cerradas).

## Requisitos

- [Rust](https://rustup.rs/) (toolchain `stable-x86_64-pc-windows-msvc`)
- Windows con Visual Studio Build Tools (para el linker de MSVC)

Las librerías de desarrollo de SDL2, SDL2_mixer y SDL2_ttf ya están incluidas en
[`vendor/sdl2/`](vendor/sdl2/) (`.lib` para enlazar, `.dll` para ejecutar) —
**no hace falta instalar SDL2 por separado**. [`build.rs`](build.rs) le indica al
linker dónde están los `.lib` y copia los `.dll` a `target/debug/` (o `target/release/`)
automáticamente en cada build, así que `cargo build` / `cargo run` funcionan directo
tras clonar, sin `.dll` sueltos en la raíz del repo.

## Cómo correr

```
cargo run
```

Para una build optimizada (recomendado para medir FPS reales):

```
cargo run --release
```

## Controles

**Gameplay**

| Tecla | Acción |
|---|---|
| W / S | Mover adelante / atrás |
| A / D | Rotar cámara |
| Mouse (movimiento horizontal) | Rotación adicional |
| Esc | Salir |

**Menús**

| Tecla | Acción |
|---|---|
| 1 / 2 / 3 | Elegir dificultad (fácil/medio/difícil) en la bienvenida |
| ENTER | Confirmar / iniciar |
| R | Reintentar (pantalla de game over) |

## Estado del proyecto

- [x] Fase 1 — Motor base: ventana SDL2, raycasting DDA (paredes tipo bloque,
      estilo Wolfenstein), movimiento + colisión + rotación (teclado y mouse),
      contador de FPS.
- [x] Fase 2 — Mapa y minimapa: laberinto 10×10 en `src/map.rs` + minimapa en
      la esquina superior derecha.
- [x] Fase 3 — Enemigo: pathfinding BFS, vida del jugador con daño por contacto,
      sprite billboard con z-buffer, estado normal/enfurecido (≤50% vida), IA
      que se congela si la ves y pausa en cruces.
- [x] Fase 4 — Sistemas y pantallas sin arte final: bienvenida (título "Not your
      house" + selector de dificultad), cinemática de introducción, HUD de vida
      en corazones, pantallas de game over/éxito, motor de audio (música con
      volumen dinámico + SFX) — todo con placeholders donde eventualmente va
      arte/audio real de Cami.
- [ ] Fase 5 — 🎨🎵 Integración de arte y audio final de Cami (reemplazo de placeholders).
- [ ] Fase 6 — Pulido, playtesting de balance, video de demo, entrega.

## Estructura

Ver la sección "Arquitectura sugerida" en [SKILL.md](SKILL.md).
