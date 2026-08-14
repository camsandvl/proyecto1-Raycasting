---
name: raycaster-horror
description: >
  Skill maestra del proyecto Ray Caster de terror en Rust para curso universitario
  (Guatemala, gráficas por computadora - parte 3). SIEMPRE usar esta skill al trabajar
  en cualquier aspecto del proyecto: motor de raycasting, colisiones, IA del enemigo,
  sistema de vida/daño, audio dinámico (SDL2 mixer), animación de sprites, diseño del
  laberinto, texturas de paredes, pantallas de bienvenida/éxito/game over, o cualquier
  decisión técnica o de arquitectura. Consultar ANTES de escribir cualquier línea de
  código Rust, definir la estructura del proyecto, o tomar cualquier decisión de stack.
  Contiene el scope exacto acordado con Cami (qué objetivos SÍ y NO se persiguen), todas
  las mecánicas de gameplay definidas, y las convenciones técnicas para Claude Code.
---

# Ray Caster de Terror (Rust) — Skill Maestra

## Contexto del proyecto

Proyecto universitario de la tercera parte del curso de gráficas por computadora.
Se entrega un ray caster simple pero jugable, con un laberinto propio (dimensiones
iguales o mayores al proporcionado por el curso — **pendiente: confirmar dimensiones
exactas del laberinto base con el enunciado del curso antes de diseñar el propio**).

Entrega: link a GitHub + video corto del software funcionando.

**Concepto:** un flat europeo sombrío — cortinas, wallpaper extraño, sensación de
encierro. Un disco suena a la distancia. Una mujer enojada, dibujada por Cami en
Procreate, persigue al jugador por el mapa como en Wolfenstein 3D.

**Stack elegido:**
- **Rust**
- **SDL2** (input, audio y ventana integrados — decisión tomada sobre minifb por la
  cantidad de mecánicas de audio dinámico que requiere el proyecto)
- Crate recomendado: `sdl2` con feature `mixer` habilitada (`sdl2 = { version = "0.37", features = ["mixer"] }`)
  para música + efectos de sonido sin librerías adicionales.

---

## ⚠️ Objetivos EXCLUIDOS explícitamente — no implementar aunque parezca fácil agregar

| Objetivo | Puntos | Razón de exclusión |
|---|---|---|
| Hardware distinto a computadora tradicional | 0–50 | Decisión de Cami: demasiado costoso en tiempo dado el resto del scope custom (arte, audio, IA) |
| Soporte de control/gamepad | +20 | Ligado al anterior, excluido |
| Selección de múltiples niveles ("mundos" distintos) | +10 | Duplicaría trabajo de arte y diseño de mapa; se prefiere un solo nivel muy pulido |
| Música de Taylor Swift | +5 extra | Cami compone su propia música original |

**Si Claude Code sugiere alguno de estos objetivos "ya que sería fácil", recordar que
fue decisión explícita del equipo NO incluirlos.**

---

## Objetivos perseguidos y puntaje esperado

| Objetivo | Puntos | Notas |
|---|---|---|
| Laberinto propio, sin atravesar paredes, sin crashear | Base | Obligatorio |
| Textura/color distinto por tipo de pared | Base | Ver sección Paredes |
| Estética del nivel (subjetivo) | 0–30 | Horror flat europeo — ver sección Ambientación |
| FPS visible en pantalla | 15 | Contador simple, meta ~15 fps |
| Cámara: adelante/atrás + rotación | 20 | Teclado |
| Rotación con mouse (solo horizontal) | +10 | |
| Minimapa en una esquina (no lado a lado) | 10 | |
| Música de fondo | 5 | Original de Cami |
| Efectos de sonido | 10 | Pasos, detección/ataque del enemigo, ambiente |
| Animación de sprite | 20 | Enemigo con 2 estados según vida del jugador |
| Pantalla de bienvenida | 5 | Arte de Cami + botón iniciar + lore breve |
| Pantalla de éxito | 10 | Al sobrevivir el tiempo límite |
| **Total perseguido** | **~135** | Nota tope real es 100, este es el buffer de seguridad |

---

## Mecánica principal: sobrevivir al enemigo

- **Condición de éxito:** el jugador debe sobrevivir el tiempo correspondiente a la
  dificultad elegida (90s / 150s / 210s, ver sección de Dificultad seleccionable más
  abajo) evitando al enemigo. Son puntos de partida, ajustables en playtesting una vez
  el maze de 10×10 esté jugable.
- **Sistema de vida:** el jugador inicia con vida completa (ej. 100). Si el enemigo lo
  alcanza, pierde vida (no es game over instantáneo) — mientras esté en rango de contacto,
  la vida baja de forma continua o en pulsos por segundo. Al llegar a 0 → game over.
- **IA del enemigo — persecución estilo Wolfenstein:**
  - El enemigo se mueve activamente por el laberinto persiguiendo al jugador, no está fijo.
  - Como el mapa tiene paredes, el movimiento directo línea-recta no sirve dentro de
    pasillos — implementar un **pathfinding simple sobre grid**: BFS (breadth-first
    search) desde la celda del jugador para generar un campo de distancias, recalculado
    cada N frames (no cada frame, por rendimiento — ej. cada 15–20 frames). El enemigo
    se mueve hacia la celda vecina con menor distancia.
  - Esto es suficiente para el nivel de un ray caster universitario — no se necesita A*
    con heurísticas ni nada más sofisticado.
- **Estados visuales del enemigo (20 pts de animación):**
  - **2 estados**: `normal` y `enfurecido`, disparados por el **porcentaje de vida del
    jugador** (no por distancia). **Umbral confirmado: vida > 50% → normal, vida ≤ 50%
    → enfurecido.**
  - Técnicamente es un sprite billboard (siempre de cara a la cámara, escalado por
    distancia, como es estándar en raycasting) que intercambia de textura según el estado.
  - **Recomendación de implementación:** para que la "animación" sea inequívoca ante
    quien califique (un simple cambio de estado podría no leerse como animación), dar a
    cada uno de los 2 estados un ciclo de 2 frames alternados (ej. leve parpadeo o
    movimiento sutil) en vez de una sola imagen estática por estado. Esto es una
    sugerencia, no bloqueante — si Cami solo entrega 2 imágenes (una por estado), el
    proyecto funciona igual, solo es menos robusto ante el criterio de "animación".

---

## Audio

Usar `sdl2::mixer` para música (`Music`) y efectos (`Chunk`).

- **Música de fondo:** track original compuesto por Cami (formato wav/mp3/ogg, cualquiera
  funciona con `sdl2::mixer`). Loop continuo durante el gameplay.
- **El disco que suena a la distancia:** el volumen sube en **saltos discretos** (no
  interpolación continua) según la distancia del enemigo al jugador, medida en celdas
  del grid (maze de 10×10). Ajustado al estilo del mapa (pasillos angostos que amortiguan
  el sonido, apartamento cerrado): se mantiene casi ambiental hasta que el enemigo está
  genuinamente cerca, para que el salto final pegue fuerte en vez de avisar con demasiada
  anticipación:
  - **Lejos** (más de 7 celdas): volumen bajo (~20%)
  - **Medio** (entre 4 y 7 celdas): volumen medio (~55%)
  - **Cerca** (menos de 4 celdas): volumen alto (~100%)

  Punto de partida — ajustar en playtesting si se siente muy silencioso o muy anticipado.

### Dificultad seleccionable (3 tiempos de supervivencia)

No es costoso de agregar — son 3 botones en la pantalla de bienvenida que fijan un valor
distinto para el timer de supervivencia, reutilizando el mismo mapa y assets (esto NO es
lo mismo que el objetivo excluido de "múltiples niveles", que exige mundos distintos):

| Dificultad | Tiempo de supervivencia |
|---|---|
| Fácil | 90 segundos |
| Medio | 150 segundos |
| Difícil | 210 segundos |

**Recomendación opcional (no obligatoria):** para que "difícil" se sienta realmente más
difícil y no solo más largo, acelerar levemente el intervalo de recálculo del pathfinding
del enemigo en esa dificultad (ej. cada 10 frames en vez de 15-20) — la lógica de BFS ya
existe, es solo cambiar un parámetro por dificultad.
- **Efectos de sonido a implementar** (10 pts):
  1. Pasos del jugador al moverse
  2. Detección/ataque del enemigo (cuando entra en rango de detección o alcanza al jugador)
  3. Ambiente de fondo (aparte de la música — ej. crujidos, viento, silencio incómodo)
- Todos los SFX y la música son producidos por Cami — Claude Code solo integra los
  archivos que ella entregue, no debe generar ni sugerir pistas de stock/libres de derechos.

---

## Ambientación y arte (estética, 0–30 pts subjetivos)

- Flat europeo sombrío: cortinas, wallpaper extraño, sensación de encierro.
- **Texturas de pared — 4 zonas temáticas del apartamento:** en vez de dibujar una
  textura única por segmento de pared (innecesario en un maze de 10×10), el mapa se
  divide en 4 zonas y cada una usa su propia textura, repetida dentro de esa zona —
  así el laberinto se siente como un apartamento real y no como un maze abstracto.
  Mapeo sobre el grid de 10×10 (filas/columnas 0-indexado):

  | Zona | Celdas del grid | Textura | Quién la dibuja |
  |---|---|---|---|
  | Recibidor (entrada) | filas 0-4, columnas 0-4 — incluye spawn del jugador (p) | Wallpaper desgastado | Cami (Procreate) |
  | Dormitorio | filas 0-4, columnas 5-9 | Cortinas pesadas | Cami (Procreate) |
  | Sala | filas 5-9, columnas 0-4 | Wallpaper floral ornamentado | Cami (Procreate) |
  | Cocina / pasillo final | filas 5-9, columnas 5-9 — incluye spawn del enemigo (e) | Textura sucia/deteriorada | Generada por código |

  Las 3 texturas de Cami se exportan como PNG y van a `assets/textures/`. La cuarta
  (cocina) se resuelve con un patrón/degradado simple por código, sin necesidad de arte
  adicional. Piso y techo: colores planos o degradado simple por código, no requieren
  dibujo a mano.
- **Enemigo:** 100% arte de Cami (pixel art o PNG hecho en Procreate), 2 estados según
  la sección de IA arriba.
- **Pantalla de bienvenida:** arte de fondo de Cami + botón "Iniciar" + un poco de
  lore/historia breve del juego (no solo título minimalista — Cami quiere contexto
  narrativo). Cami dirige el diseño visual, Claude Code organiza el layout/código.
- **Pantalla de éxito:** se muestra al sobrevivir el tiempo de la dificultad elegida
  (90/150/210s). Reutiliza elementos
  visuales de la bienvenida (misma paleta/tipografía) con un mensaje de cierre a la
  historia — diseño exacto pendiente de que Cami aporte arte, pero la estructura
  (fondo + texto + volver a jugar) ya está definida.
- **Pantalla de game over — freeze-frame con jumpscare:** al momento en que la vida del
  jugador llega a 0, el juego congela el último frame renderizado (freeze-frame del
  instante de la muerte) y encima dibuja, casi a pantalla completa, una imagen dedicada
  de la cara del enemigo en close-up extremo (como si se hubiera acercado muchísimo a la
  cámara) — **NO es el mismo sprite que se usa durante el gameplay**, es un asset
  independiente que Cami dibuja específicamente para este momento, con una versión más
  perturbadora/detallada de la cara. Encima de eso: un overlay de color oscuro semi-
  transparente leve, y el texto "GAME OVER" centrado.
  - **Asset nuevo requerido:** `assets/ui/jumpscare_face.png` — ilustración de Cami,
    close-up de la cara del enemigo, pensada para casi llenar la pantalla (no billboard
    3D como el sprite de gameplay, es un asset 2D en espacio de pantalla).
  - Implementación: al detectar vida = 0, dejar de actualizar el mundo 3D (no renderizar
    más frames del raycaster), dibujar el `jumpscare_face.png` sobre el freeze-frame,
    aplicar el overlay oscuro, luego el texto y botón de reintentar.

---

## Arquitectura sugerida del proyecto (Rust)

```
raycaster-horror/
├── Cargo.toml
├── assets/
│   ├── textures/       ← PNGs de paredes (mezcla Cami + generadas)
│   ├── sprites/
│   │   └── enemy/       ← estados normal/enfurecido (+ frames de animación si aplica)
│   ├── audio/
│   │   ├── music/       ← track de fondo de Cami
│   │   └── sfx/         ← pasos, detección/ataque, ambiente
│   └── ui/
│       ├── jumpscare_face.png  ← close-up del enemigo para game over (asset aparte del sprite de gameplay)
│       └── ...                  ← arte de pantallas de bienvenida/éxito
├── src/
│   ├── main.rs
│   ├── engine/
│   │   ├── raycasting.rs   ← algoritmo DDA para detección de paredes
│   │   ├── camera.rs       ← movimiento adelante/atrás, rotación teclado+mouse
│   │   └── minimap.rs
│   ├── entities/
│   │   ├── player.rs       ← posición, vida, colisiones
│   │   └── enemy.rs        ← pathfinding BFS, estado según vida del jugador
│   ├── audio/
│   │   └── mixer.rs        ← música + SFX + volumen dinámico del disco
│   ├── render/
│   │   ├── walls.rs        ← texturas por tipo de pared
│   │   ├── sprites.rs       ← billboard rendering del enemigo (con z-buffer)
│   │   └── hud.rs           ← FPS, minimapa
│   ├── screens/
│   │   ├── welcome.rs
│   │   ├── success.rs
│   │   └── game_over.rs    ← freeze-frame + jumpscare_face.png + overlay + texto
│   └── map.rs               ← definición del laberinto (grid + tipos de pared)
└── README.md
```

---

## Controles

- **W / S** — mover adelante / atrás
- **A / D** — rotar cámara (teclado)
- **Mouse (movimiento horizontal)** — rotación adicional
- Colisión: el jugador nunca debe poder atravesar una pared ni el juego debe crashear
  al chocar con los límites del mapa — validar movimiento contra el grid antes de aplicar
  la posición nueva.

---

## Checklist de puntos (objetivo ~135, tope real 100)

| Ítem | Pts | Estado |
|---|---|---|
| Laberinto propio + colisiones sin crash | Base | ⚙️ Obligatorio |
| Texturas/colores distintos por pared | Base | 🎨 Mixto (Cami + generado) |
| Estética horror europea | 0–30 | 🎨 |
| FPS counter | 15 | ⚙️ |
| Cámara adelante/atrás + rotación teclado | 20 | ⚙️ |
| Rotación con mouse | 10 | ⚙️ |
| Minimapa en esquina | 10 | ⚙️ |
| Música de fondo original | 5 | 🎵 Cami |
| Efectos de sonido (3 tipos) | 10 | 🎵 Cami |
| Animación de sprite (2 estados por vida) | 20 | 🎨 Cami + ⚙️ |
| Pantalla de bienvenida (arte + lore) | 5 | 🎨 Cami + ⚙️ |
| Pantalla de éxito | 10 | ⚙️ |
| **Total** | **~135** | |

---

## Diseño del laberinto (10×10, con loops para evasión)

El laberinto de referencia de la clase es 4×4 y **perfecto** (un solo camino posible
entre dos celdas cualesquiera) — eso es malo para un juego de evasión porque el enemigo
puede acorralar al jugador en callejones sin salida. Este diseño es **10×10** (más del
doble de dimensión que el de referencia, cumple "iguales o mayores") y se le removió
intencionalmente ~14% de las paredes internas de un maze perfecto generado por DFS, para
crear loops — el jugador siempre tiene una ruta alterna para rodear al enemigo.

```
+--+--+--+--+--+--+--+--+--+--+
| p                           |
+--+  +--+--+  +--+  +--+  +  +
|  |  |     |     |     |     |
+  +  +  +  +  +  +  +  +  +--+
|  |     |     |  |     |     |
+  +--+--+--+  +  +  +  +--+--+
|           |  |     |        |
+  +--+  +--+  +  +--+  +--+  +
|     |     |           |     |
+--+--+--+  +--+--+--+  +  +  +
|           |        |  |  |  |
+  +--+--+  +  +--+  +  +  +  +
|     |           |  |  |     |
+  +  +--+  +--+--+  +  +--+  +
|        |     |     |        |
+  +--+  +--+  +  +  +--+--+  +
|     |  |        |  |        |
+--+  +  +--+--+--+--+  +  +--+
|     |                      e|
+--+--+--+--+--+--+--+--+--+--+
```

`p` = spawn del jugador (esquina superior izquierda). `e` = spawn del enemigo (esquina
inferior derecha, lejos del jugador para dar un margen inicial antes de que empiece la
persecución).

Convertir esta representación a la estructura de datos del grid en `map.rs` (walls por
celda: N/S/E/W según los segmentos `--` y `|` de arriba/izquierda de cada celda).

---

## Decisiones cerradas (ya no quedan pendientes abiertos)

| Parámetro | Valor definido |
|---|---|
| Dimensión del laberinto | 10×10 (con loops, ver arriba) |
| Umbral de vida para estado "enfurecido" del enemigo | ≤ 50% de vida |
| Volumen del disco — lejos / medio / cerca | >7 celdas (20%) / 4–7 celdas (55%) / <4 celdas (100%) |
| Tiempo de supervivencia para ganar | Seleccionable: fácil 90s / medio 150s / difícil 210s |
| Pantalla de game over | Freeze-frame + cara del enemigo en close-up (jumpscare_face.png) + overlay oscuro + texto |
| Texturas de pared | 4 zonas temáticas: 3 dibujadas por Cami + 1 generada por código (ver mapeo de zonas) |

Todos estos números son puntos de partida razonables, no reglas rígidas — cualquiera se
ajusta con un cambio pequeño una vez haya algo jugable para probar.

---

## Flujo de trabajo recomendado

### Fase 1 — Motor base (sin arte aún)
1. Setup del proyecto Rust + SDL2, ventana básica
2. Algoritmo de raycasting (DDA) con paredes de un solo color de placeholder
3. Movimiento del jugador + colisiones + rotación (teclado y mouse)
4. FPS counter en pantalla

### Fase 2 — Mapa y minimapa
1. Diseñar el laberinto propio (grid, respetando dimensiones mínimas)
2. Minimapa en una esquina

### Fase 3 — Enemigo
1. Pathfinding BFS básico sobre el grid
2. Sistema de vida del jugador + daño por contacto
3. Sprite billboard con z-buffer (placeholder al inicio)
4. Lógica de estado (normal/enfurecido) según vida

### Fase 4 — 🎨 Integración de arte (Cami)
1. Texturas de pared (mixtas)
2. Sprites del enemigo (2 estados)
3. Arte de pantallas de bienvenida/éxito/game over

### Fase 5 — 🎵 Audio
1. Integrar música de fondo con loop
2. Volumen dinámico en 3 saltos según distancia
3. Efectos de sonido (pasos, detección/ataque, ambiente)

### Fase 6 — Pantallas y pulido
1. Pantalla de bienvenida con lore + selector de dificultad (fácil/medio/difícil)
2. Pantalla de éxito (condición: sobrevivir el tiempo de la dificultad elegida)
3. Pantalla de game over (freeze-frame + jumpscare_face.png)
4. Playtesting de balance (vida, tiempos por dificultad, distancias de audio)
5. Grabar video de demo + README + push a GitHub

---

## Notas de implementación importantes

- **DDA (Digital Differential Analyzer):** es el algoritmo estándar para raycasting tipo
  Wolfenstein — evitar reinventar con approach de fuerza bruta por rendimiento.
- **Z-buffer para sprites:** necesario para que el enemigo se oculte correctamente detrás
  de paredes más cercanas al renderizar.
- **BFS recalculado, no por frame:** recalcular el campo de distancias del pathfinding
  cada 15–20 frames (no cada frame) es suficiente para que la persecución se vea fluida
  sin gastar rendimiento — importante para mantener los ~15 fps mínimos requeridos.
- **No inventar mecánicas no acordadas:** si algo no está en este documento (ej. tipos de
  arma, múltiples enemigos, power-ups), no agregarlo sin confirmar con Cami primero.
