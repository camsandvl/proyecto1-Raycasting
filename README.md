# Ray Caster de Terror

Proyecto universitario (gráficas por computadora, parte 3): un ray caster estilo
Wolfenstein 3D ambientado en un flat europeo sombrío, con un enemigo que persigue al
jugador por un laberinto propio de 10×10. Ver [SKILL.md](SKILL.md) para el diseño
completo (scope, mecánicas, decisiones cerradas).

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

| Tecla | Acción |
|---|---|
| W / S | Mover adelante / atrás |
| A / D | Rotar cámara |
| Mouse (movimiento horizontal) | Rotación adicional |
| Esc | Salir |

## Estado del proyecto

- [x] Fase 1 — Motor base: ventana SDL2, raycasting DDA (paredes tipo bloque,
      estilo Wolfenstein), movimiento + colisión + rotación (teclado y mouse),
      contador de FPS.
- [x] Fase 2 — Mapa y minimapa: laberinto 10×10 en `src/map.rs` + minimapa en
      la esquina superior derecha.
- [x] Fase 3 — Enemigo: pathfinding BFS (recalculado cada 18 frames), vida del
      jugador con daño por contacto, sprite billboard con z-buffer, estado
      normal/enfurecido (≤50% vida) con parpadeo de 2 frames.
- [ ] Fase 4 — Integración de arte (Cami): texturas, sprites, pantallas.
- [ ] Fase 5 — Audio: música, volumen dinámico, efectos de sonido.
- [ ] Fase 6 — Pantallas de bienvenida/éxito/game over, pulido, playtesting, demo.

## Estructura

Ver la sección "Arquitectura sugerida" en [SKILL.md](SKILL.md).
