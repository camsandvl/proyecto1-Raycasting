# Proyecto 1

Ray caster estilo Wolfenstein 3D 

## Tecnologías

- **Rust**
- **SDL2** — ventana, input (teclado + mouse) y render
- **SDL2_image** — carga de texturas y fondos en PNG
- **SDL2_ttf** — texto (HUD, títulos, menús)
- **SDL2_mixer** — música de fondo y efectos de sonido

## Cómo correr el proyecto

Requisitos:
- [Rust](https://rustup.rs/) (toolchain `stable-x86_64-pc-windows-msvc`)
- Windows con Visual Studio Build Tools (linker de MSVC)

Las librerías de SDL2 ya vienen incluidas en [`vendor/sdl2/`](vendor/sdl2/)
(`.lib` para enlazar, `.dll` para ejecutar) — no hace falta instalar SDL2 por
separado. `cargo build` / `cargo run` funcionan directo tras clonar el repo.

```
cargo run
```

Build optimizada (recomendada para medir FPS reales):

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
| ENTER | Confirmar / iniciar / volver al menú |
| R | Reintentar o volver al menú |

## Objetivos cumplidos

| Objetivo | Puntos | Estado |
|---|---|---|
| Laberinto propio, sin atravesar paredes, sin crashear | Base | ✅ |
| Textura/color distinto por tipo de pared | Base | ✅ |
| Estética del nivel | 0–30 | ✅ |
| FPS visible en pantalla | 15 | ✅ |
| Cámara: adelante/atrás + rotación (teclado) | 20 | ✅ |
| Rotación con mouse (solo horizontal) | +10 | ✅ |
| Minimapa en una esquina | 10 | ✅ |
| Música de fondo | 5 | ✅ |
| Efectos de sonido | 10 | ✅ |
| Animación de sprite (2 estados) | 20 | ✅ |
| Pantalla de bienvenida | 5 | ✅ |
| Pantalla de éxito | 10 | ✅ |
| Hardware distinto a computadora tradicional | 0–50 | ❌ No implementado |
| Soporte de control/gamepad | +20 | ❌ No implementado |
| Selección de múltiples niveles | +10 | ❌ No implementado (hay selector de dificultad sobre el mismo mapa, no niveles distintos) |
| Música de Taylor Swift | +5 | ❌ No implementado |
