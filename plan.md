# Coquillage — plan de desarrollo

Documento vivo. Aquí va el prompt original, la referencia visual, qué hay que hacer y el estado real del repo.

**Workspace:** `C:\Users\mario\workplace\Coquillage`  
**Repo:** https://github.com/MarioJoseMelendez/Coquillage  
**Motor:** Rust directo, sin Godot / Unity / Bevy.  
**Fase actual:** 8 — Player (compila; falta tu OK visual)

---

## Qué hacer

1. Completar **una sola fase** del roadmap, verificar, parar y esperar aprobación.
2. Empezar por **Phase 1**. No lighting avanzado, no agua, no bosque procedural, no clima.
3. Toda decisión de iluminación y tamaño de sprites se ancla a la imagen de referencia de abajo (mood y escala), con cámara **top-down ortográfica estricta** (la referencia es 3/4; no copiamos ese ángulo).
4. Guardar el progreso de cada fase en este archivo.
5. Pixler MCP: instalarlo cuando exista el token `pxl_live_...` en `~/.cursor/mcp.json`. Phase 1 usa gráficos de prueba, no assets de Pixler.
6. **Cobertura / fade:** si el jugador queda debajo de un objeto que lo tapa (copa de árbol, arbusto grande), **solo ese objeto** se vuelve semitransparente (efecto de invisibilidad / ghost). El resto del bosque no cambia. Sin blur: es alfa, los píxeles siguen nítidos. **Apagado hasta que exista el jugador:** el probe en el centro de la cámara ghosteaba lo que mirabas (copa semitransparente + sombra opaca = “roca y arbusto” pegados al centro).

### Ahora mismo (Phase 8 — player)

- Phase 1–7 **aprobadas** (fundación visual).
- Jugador top-down 16×16, WASD, colisión agua/troncos, cámara follow, fade de copa.
- Parar y esperar OK visual. No Phase 9.

### Fases que faltan (después de la 7)

El spec original **terminaba en Phase 7**. Lo que sigue es gameplay, numerado ahora:

| Fase | Qué añade |
| --- | --- |
| **8 Player** (esta) | Avatar, caminar, colisión 16×16, cámara, ghost de copa |
| **9 Audio** | Lluvia / viento / pasos (`kira` si hace falta) |
| Más adelante (cuando lo pidas) | Capas subterráneas, antorcha puntual, assets Pixler |

### Herramientas previstas (evaluar al usarlas)

| Crate | Para qué | Cuándo |
| --- | --- | --- |
| `wgpu` | GPU / render | Phase 1 |
| `winit` | ventana e input | Phase 1 |
| `glam` | matrices ortográficas | Phase 1 |
| `bytemuck` | uniforms GPU | Phase 1 |
| `pollster` | init async de wgpu | Phase 1 |
| `image` | cargar PNG | Phase 2 |
| `kira` | audio | solo si hace falta audio |

Toolchain local ya vista: `rustc` / `cargo` 1.96.0.

---

## Referencia visual

**Primarias (mandan, en este orden):**

1. [`assets/reference/minecraft-hd-lighting.jpg`](assets/reference/minecraft-hd-lighting.jpg) — sol cálido, hojas en manchas, sombras frías. También el mood del shader.
2. [`assets/reference/wilderness-lighting-sprites.jpg`](assets/reference/wilderness-lighting-sprites.jpg) — bosque HD, sendero, dosel.
3. [`assets/reference/slynyrd-conifer-topdown.png`](assets/reference/slynyrd-conifer-topdown.png) — método de pino top-down: **módulos de follaje apilados**, no radios.

**Apoyo (forma de copa, no cámara):** [`topdown-map-canopies.png`](assets/reference/topdown-map-canopies.png). El sheet 3/4 [`tree-sheet-34-view.png`](assets/reference/tree-sheet-34-view.png) solo para densidad; no copiar alzado ni el asset. Construcción Slynyrd 160: [`slynyrd-isometric-tree.png`](assets/reference/slynyrd-isometric-tree.png) — anillos de color, copa de brócoli, tocón+raíces; **no** el ángulo isométrico.

**Qué no repetir:** [`current-look-avoid.png`](assets/reference/current-look-avoid.png) — árboles-estrella, hongos de confeti, rejilla naranja.

Biblia de color / anti-slop: [`assets/reference/pixel-art-bible.md`](assets/reference/pixel-art-bible.md) (Saint11, hue shift, Essence24 / Purple Sunset, Slynyrd).

La cámara sigue **top-down ortográfica estricta**. De las refs se copia paleta, luz, tamaños y método de copa, no el ángulo 3/4.

### Iluminación (de la imagen → Phase 4+, no implementar ahora)

- Sol alto, ligeramente desplazado; ejes de luz cálidos (amarillo-blanco) desde el tercio superior.
- God rays / shafts volumétricos colando entre copas.
- Marco casi negro de hojas: oclusión de dosel, viñeta natural.
- Suelo en sombra: verde-azul frío, saturación baja.
- Sendero iluminado: marrones cálidos, adoquines con highlight de humedad.
- Luz local de antorcha: naranja puntual, bloom corto (lenguaje de luz, no gameplay todavía).
- Lluvia: trazos finos que caen (Phase 6).
- No fotorealismo 3D. La geometría pixel-art sigue nítida; la luz es una capa aparte a resolución de display.

### Tamaños de sprites (de la imagen + tabla del spec)

Resolución lógica prevista: **640×360**. Tile base: **32×32**. Upscale nearest-neighbor a escala entera (2× en 1280×720, 3× en 1920×1080).

| Tipo | Tamaño | Notas desde la referencia |
| --- | --- | --- |
| Tile de terreno | 32×32 | sendero ~3–4 tiles de ancho; adoquines internos ~16×16 |
| Hierba / flores / hongos | 16×16 a 32×32 | clusters pequeños, no un sprite de “césped plano” |
| Arbustos / helechos | 32×32 a 64×64 | silueta irregular, varias densidades |
| Rocas / tocones | 32×32 a 64×64 | musgo, corteza, no cubos |
| Copas de árbol (top-down) | 64×96 a 96×128 o más | en la ref cubren gran parte del encuadre; no forzarlas a un tile |
| Criatura media | 32×32 a 48×48 | el ser naranja del camino |
| Figura humana (fuera de scope inicial) | ~24×32 a 32×48 | solo para calibrar escala; Phase 1–3 no meten NPCs |

Sprites aislados: fondo transparente, sin sombra ni lighting horneados (la sombra irá en la capa ambiental). Varias variantes por categoría. Densidad de píxel constante aunque el canvas cambie de tamaño.

Lenguaje de textura: **Minecraft HD** (Faithful / vanilla 32px) + hue shift. Pocas formas grandes, paleta corta. Sombras **moradas**, luces amarillo-lima. Cero sal de píxeles. Árbol top-down = pads / manchas de copa (Slynyrd), tronco entre capas, **prohibido** rueda de radios.

### Cobertura del jugador (obligatorio)

El juego **detecta** cuando el jugador está debajo de un objeto que lo tapa (copa, arbusto grande, más adelante techo de copa densa).

- Solo **ese** instance se desvanece (alfa ~0.3, ghost). No se borra.
- El resto de árboles no se tocan.
- Sin blur ni shaders de distorsión: el pixel-art sigue nítido.
- Hitbox más chica que el sprite (centro de la copa), para no fadealizar al rozar el borde.
- Hoy el probe es el **centro de la cámara**. El día que exista el sprite del jugador, el probe pasa a su posición.

### Paleta anclada a la imagen

- Verdes bosque profundos, oliva, amarillo-verde apagado en hojas tocadas por el sol.
- Marrones tierra y sendero húmedo.
- Grises piedra con highlight cálido.
- Azul-verde frío en sombra y lluvia.
- Sol: amarillo sucio, no neón. Luces giran hacia amarillo; sombras hacia **morado**.
- Evitar verdes chillones, sombreado a gris, y saturación alta en el highlight.

---

## Prompt original (spec del proyecto)

Texto de referencia para el trabajo. Las fases de abajo mandan. No implementar el spec entero de una vez.

---

Project Specification — High-Definition Pixel Art Wilderness Sandbox

You are developing the visual and technical foundation of a top-down wilderness sandbox game.

The project must be implemented directly in Rust, without using an existing game engine.

The objective is to create a visually rich, explorable natural world that combines high-definition pixel art with sophisticated, realistic environmental rendering.

This document defines the artistic direction, technical requirements, architecture guidelines and development roadmap.

IMPORTANT: Development must happen progressively, one phase at a time. Do not attempt to implement the entire project in a single pass.

### 1. Core Vision

Create a visually rich, immersive wilderness sandbox game with a strict top-down orthographic perspective.

The world should combine the visual identity of high-definition pixel art with realistic environmental rendering.

The intended aesthetic is a detailed, living natural environment where every object is carefully pixel-crafted, while lighting, shadows, water, atmosphere and environmental effects behave in a visually realistic way.

The world should feel organic, quiet, immersive and explorable.

Primary inspiration:

- The environmental freedom and exploration of Minecraft.
- The top-down perspective and environmental presentation of GTA: Chinatown Wars.
- Modern high-definition pixel art with sophisticated environmental lighting.
- Realistic natural environments translated into a coherent pixel-art visual language.

Do not imitate the exact visual identity of any existing game.

Current Scope

For the initial development, focus exclusively on natural environments.

The initial world should contain:

- Forests.
- Grasslands.
- Rivers.
- Lakes.
- Ponds.
- Rocks.
- Trees.
- Bushes.
- Natural terrain.
- Other environmental elements.

Do not introduce cities, buildings, vehicles, human characters, NPCs, weapons or urban elements.

The first objective is to establish the visual foundation of the wilderness.

### 2. Camera and Perspective

- Strict top-down orthographic 2D perspective.
- Camera looks directly down at the environment.
- No isometric perspective.
- No perspective convergence.
- No 3D-looking object presentation.
- Objects must maintain consistent scale and orientation.
- Trees, rocks, vegetation and environmental objects must be readable from above.
- The environment should support continuous exploration in all directions.

The camera should make navigation and spatial awareness intuitive.

### 3. Pixel Art Resolution and Density

The art style must use high-definition, fine-grained pixel art.

Avoid traditional chunky 8-bit or 16-bit aesthetics.

Requirements:

- Small, carefully controlled pixel clusters.
- High detail density.
- Crisp pixel boundaries.
- Carefully constructed silhouettes.
- Fine texture variation.
- No excessive pixelation.
- No blurry or artificially smoothed sprite textures.
- Consistent pixel density across all assets.

Suggested technical reference:

| Asset type | Suggested dimensions |
| --- | --- |
| Base terrain tiles | 32×32 pixels |
| Small environmental objects | 16×16 to 32×32 pixels |
| Medium objects | 32×32 to 64×64 pixels |
| Large trees | 64×96 to 96×128 pixels or larger |

These dimensions are guidelines, not rigid restrictions.

Do not force large objects into small tiles.

Maintain consistent pixel scale even when assets have different dimensions.

Rendering Resolution

The internal resolution and sprite dimensions must be independent concepts.

Requirements:

- The pixel-art world should render at a controlled logical resolution.
- The pixel-art layer must preserve crisp pixel boundaries.
- Upscaling must use nearest-neighbor sampling.
- Avoid unintended smoothing and texture interpolation.
- Environmental effects should be capable of rendering at native display resolution.
- The rendering pipeline must support different resolutions for pixel-art content and environmental effects.

Do not assume that the entire game must render at a low resolution.

The intention is to preserve detailed pixel-art objects while allowing smooth, sophisticated environmental rendering.

### 4. Color Palette

Use a natural, restrained and realistic color palette.

Primary colors:

- Multiple shades of natural green.
- Deep forest greens.
- Olive and muted yellow-green.
- Earthy browns.
- Dark soil tones.
- Natural stone grays.
- Muted blues and turquoise for water.
- Subtle warm sunlight tones.

Color requirements:

- Avoid excessive saturation.
- Avoid artificial neon colors.
- Avoid overly bright greens.
- Use natural variation between neighboring surfaces.
- Use color temperature to communicate lighting conditions.
- Maintain sufficient contrast between interactive objects and their surroundings.

Different vegetation species should have distinguishable color characteristics.

### 5. Terrain and Ground Materials

Terrain must feel detailed, irregular and naturally distributed.

Required terrain types:

Grass

- Multiple grass variations.
- Small individual grass clusters.
- Irregular patches.
- Different shades and densities.
- Occasional flowers and small plants.
- Natural transitions between grass and dirt.

Dirt

- Fine soil texture.
- Small stones.
- Organic irregularities.
- Footpaths and worn ground.
- Subtle color variation.

Sand

- Fine grain-like texture.
- Natural tonal variation.
- Irregular boundaries.
- Transitions into soil and water.

Stone

- Distinct rock patterns.
- Cracks and surface irregularities.
- Multiple stone sizes.
- Moss and environmental wear.

Forest Floor

- Fallen leaves.
- Twigs.
- Roots.
- Moss.
- Small plants.
- Organic debris.

Avoid repetitive tile patterns and obvious grid-like terrain distribution.

### 6. Vegetation and Natural Objects

Vegetation is one of the most important visual elements.

Trees

- Multiple species.
- Different canopy shapes.
- Irregular silhouettes.
- Dense and sparse foliage.
- Natural color variation.
- Detailed canopy texture.
- Subtle depth and shadowing.
- Different sizes and growth patterns.

Bushes

- Different sizes.
- Irregular silhouettes.
- Multiple foliage densities.

Other Vegetation

- Flowers.
- Ferns.
- Tall grass.
- Small plants.
- Mushrooms.
- Roots.
- Fallen branches.

Vegetation should never look like repeated copies of the same sprite.

Generate multiple variations for each object category.

### 7. Water Rendering

Water should be one of the most visually sophisticated elements.

Requirements:

- Smooth animated surface movement.
- Natural reflections.
- Subtle transparency.
- Refraction-like distortion.
- Small ripples.
- Different water depths.
- Shoreline transitions.
- Animated highlights.
- Environmental color variation.

Water must preserve the overall pixel-art aesthetic while allowing higher-resolution shader effects.

Avoid simplistic repeating blue textures.

### 8. Lighting and Shaders

Lighting should be realistic without destroying pixel-art geometry.

Use a separate rendering resolution for environmental effects when appropriate.

Required effects:

Global Illumination Approximation

- Ambient environmental lighting.
- Natural light distribution.
- Subtle indirect illumination.

Shadows

- Soft environmental shadows.
- Directional sunlight.
- Tree canopy shadows.
- Object-to-ground contact shadows.
- Consistent light direction.

Ambient Occlusion

- Subtle darkening around objects.
- Natural depth perception.
- Avoid excessive black outlines.

Sunlight

- Directional light.
- Warm and cool color temperature variation.
- Light filtering through vegetation.
- Subtle brightness variation.

Atmospheric Effects

- Environmental fog.
- Distance-based atmospheric fading.
- Subtle haze.
- Optional light shafts through dense vegetation.

All effects should be restrained and coherent.

Do not make the scene look like a photograph or a conventional 3D game.

### 9. Weather and Environmental Effects

The environment should eventually support dynamic weather.

Potential effects:

- Rain.
- Wet ground.
- Water puddles.
- Wind affecting vegetation.
- Moving tree shadows.
- Fog.
- Atmospheric haze.
- Changing sunlight intensity.

Rain should interact visually with water surfaces and ground materials.

Wet surfaces may become slightly darker and more reflective.

Wind should create subtle movement without making every object constantly animate.

### 10. Environmental Composition

The world must feel naturally distributed rather than artificially assembled.

Requirements:

- Irregular forest density.
- Natural clearings.
- Small lakes and ponds.
- Rivers and streams.
- Rocky areas.
- Open grasslands.
- Dense vegetation clusters.
- Natural transitions between biomes.

Avoid perfectly symmetrical arrangements.

Avoid obvious repeating patterns.

Use variation in object density, scale and distribution.

The environment should support procedural generation without looking procedurally repetitive.

### 11. Animation

Environmental animation should be subtle.

Examples:

- Moving grass.
- Swaying foliage.
- Water movement.
- Animated flowers.
- Rainfall.
- Floating particles.
- Subtle lighting changes.

Animations must preserve the original sprite's pixel density and visual identity.

Avoid excessive frame interpolation or blur.

### 12. Asset Production Rules

Every asset must follow the same visual language.

When generating assets:

- Use transparent backgrounds for isolated sprites.
- Preserve consistent pixel density.
- Avoid baked-in shadows when shadows will be generated dynamically.
- Avoid baked-in lighting when dynamic lighting is expected.
- Keep sprites compatible with sprite atlases and texture arrays.
- Produce multiple variations where repetition would be visible.
- Use coherent color palettes.
- Maintain clean silhouettes.

Tilesets must include appropriate edge and transition variants.

Do not introduce inconsistent resolutions or unrelated artistic styles.

Do not generate assets independently without checking how they look together in an actual environment.

### 13. Technical Architecture — Rust Without a Game Engine

The project must be implemented directly in Rust.

Do not use Godot, Unity, Bevy or another complete game engine.

A lightweight custom rendering architecture is preferred.

Suggested Technologies

- Rust: primary programming language.
- wgpu: GPU abstraction and rendering.
- winit: window management and input events.
- WGSL: custom GPU shaders.
- image: image loading and asset processing.
- glam: vector and matrix mathematics.
- Kira: optional audio library, only when audio implementation becomes necessary.

These are suggested tools, not mandatory dependencies.

Evaluate dependencies before introducing them.

Avoid unnecessary libraries and excessive abstraction.

Rendering Architecture

Separate the rendering system into distinct layers.

A. Pixel-Art World Layer

Responsible for:

- Terrain.
- Trees.
- Rocks.
- Vegetation.
- Environmental objects.
- Sprite animations.

This layer should preserve the intended pixel density and crisp visual appearance.

B. Environmental Rendering Layer

Responsible for:

- Lighting.
- Shadows.
- Ambient occlusion.
- Water effects.
- Fog.
- Weather.
- Environmental post-processing.

This layer may render at native display resolution while the world layer maintains its own logical resolution.

C. Presentation Layer

Responsible for:

- Combining rendering layers.
- Final compositing.
- Upscaling.
- Display output.

Avoid indiscriminate smoothing of the pixel-art layer.

Code Organization

Keep the project modular and maintainable.

Suggested initial organization:

- `src/main.rs`
- `src/app/`
- `src/renderer/`
- `src/camera/`
- `src/world/`
- `src/assets/`
- `src/math/`
- `shaders/`
- `assets/`
- `ART_DIRECTION.md`

The structure may evolve as the implementation requires.

Do not create empty modules or unnecessary abstractions prematurely.

Keep individual functions reasonably small and responsibilities clear.

Performance

- Use GPU rendering for the main rendering pipeline.
- Avoid unnecessary CPU-side pixel processing every frame.
- Avoid excessive draw calls.
- Use texture atlases where appropriate.
- Avoid unnecessary memory allocations in the render loop.
- Keep the architecture scalable for larger environments.
- Profile before introducing complex optimizations.

The target is smooth performance on ordinary desktop hardware, including systems without dedicated GPUs.

### 14. Development Roadmap — Strict Phased Execution

Development must follow the phases below.

Complete only one phase at a time.

Do not implement future phases prematurely.

After each phase, run the project, verify the result, report what works and stop for approval.

#### Phase 1 — Rendering Foundation

Objective: establish a working custom Rust rendering foundation.

Tasks:

- Inspect the existing repository before making changes.
- Understand the existing structure and constraints.
- Establish the Rust application architecture.
- Configure the window and event loop.
- Initialize GPU rendering.
- Implement the camera and basic top-down orthographic projection.
- Establish the pixel-art rendering pipeline.
- Define a consistent logical pixel density.
- Implement nearest-neighbor upscaling.
- Establish the architecture needed to support a separate environmental rendering layer later.
- Render a basic terrain scene with clearly identifiable test graphics.
- Verify correct scaling, alignment and pixel sharpness.

Deliverable:

A working application displaying a basic top-down pixel-art environment.

Acceptance criteria:

- The application compiles and runs.
- The window opens correctly.
- The camera works.
- The rendering pipeline produces a stable image.
- Pixel boundaries remain crisp when upscaled.
- The scene does not blur or distort when the window is resized.
- The architecture can accommodate additional rendering layers.

Do not implement advanced lighting, water, procedural forests or weather in this phase.

Stop after Phase 1 and wait for approval.

#### Phase 2 — Terrain and Asset Pipeline

Objective: establish the asset pipeline and basic terrain rendering.

Tasks:

- Implement sprite and texture loading.
- Establish texture atlases where appropriate.
- Implement terrain rendering.
- Support different terrain materials.
- Establish asset metadata and organization.
- Support different sprite dimensions without inconsistent pixel density.
- Implement basic terrain transitions.
- Test asset consistency and visual quality.

Deliverable:

A coherent terrain scene with grass, dirt, stone and sand.

Acceptance criteria:

- Assets load reliably.
- Different terrain materials are distinguishable.
- Terrain transitions work.
- Pixel density remains consistent.
- Assets can be added without modifying the rendering architecture.

Stop after Phase 2 and wait for approval.

#### Phase 3 — Vegetation and Environmental Composition

Objective: create a visually convincing natural environment.

Tasks:

- Implement trees, bushes, grass and other vegetation.
- Introduce multiple asset variations.
- Implement environmental object placement.
- Introduce natural density variation.
- Support irregular natural compositions.
- Test different environmental arrangements.

Deliverable:

A visually convincing top-down forest environment.

Acceptance criteria:

- Vegetation has consistent visual style.
- Repetition is minimized.
- Objects maintain correct scale and pixel density.
- The forest feels organic rather than grid-based.

Stop after Phase 3 and wait for approval.

#### Phase 4 — Lighting and Shadows

Objective: introduce sophisticated environmental lighting.

Tasks:

- Implement environmental lighting.
- Add directional sunlight.
- Implement soft shadows.
- Add ambient occlusion approximations.
- Introduce contact shadows.
- **Sombras dinámicas de vegetación:** pasto corto vs pasto alto. Un uniform `sun_dir` desplaza manchas moradas en el suelo; el pasto alto proyecta un blob más largo al SE, el corto uno chico. Árboles y arbustos usan el mismo sistema (hoy las sombras de contacto están horneadas / instanciadas fijas). Sin blur 3D: sprites o atlas de sombra nearest-neighbor.
- Separate lighting resolution from pixel-art resolution where appropriate.
- Ensure lighting does not compromise pixel-art geometry.

Deliverable:

A forest scene with sophisticated, realistic environmental lighting.

Acceptance criteria:

- Light direction is coherent.
- Shadows respond appropriately to objects.
- Pixel-art textures remain crisp.
- Lighting creates depth without making the world look 3D.

Stop after Phase 4 and wait for approval.

Use the reference image as the lighting target: warm shafts through canopy, cool shadow wells, wet-path highlights, optional torch as a point light later. Do not copy photographic 3D.

#### Phase 5 — Water and Atmospheric Effects

Objective: create sophisticated environmental rendering.

Tasks:

- Implement animated water.
- Add reflections, ripples and distortion.
- Introduce shoreline effects.
- Add fog and atmospheric effects.
- Test different rendering resolutions for environmental effects.
- Ensure environmental shaders remain visually consistent with the art direction.

Deliverable:

A natural environment with convincing water and atmospheric rendering.

Acceptance criteria:

- Water animation is smooth.
- Water integrates with surrounding terrain.
- Atmospheric effects do not blur the pixel-art world.
- Effects remain performant.

Stop after Phase 5 and wait for approval.

#### Phase 6 — Environmental Animation and Weather

Objective: make the environment feel alive.

Tasks:

- Implement wind.
- Animate vegetation.
- Add environmental particles.
- Implement rain.
- Implement wet-surface effects.
- Introduce weather-related lighting variations.
- Add subtle atmospheric animation.

Deliverable:

A living, animated wilderness with environmental variation.

Acceptance criteria:

- Animations are subtle and coherent.
- Vegetation responds naturally to wind.
- Rain interacts visually with the environment.
- No unnecessary animation artifacts appear.

Stop after Phase 6 and wait for approval.

The reference image’s rain streaks and wet cobble highlights are the weather target.

#### Phase 7 — Visual Consistency and Optimization

Objective: consolidate and optimize the visual foundation.

Tasks:

- Review all assets against art direction in this plan / `ART_DIRECTION.md` if it exists.
- Identify inconsistent pixel densities.
- Fix visual artifacts.
- Review rendering architecture.
- Optimize GPU rendering where necessary.
- Profile performance.
- Reduce unnecessary allocations and draw calls.
- Remove redundant code and dependencies.
- Document important rendering decisions.

Deliverable:

A cohesive, optimized visual foundation ready for subsequent gameplay development.

Acceptance criteria:

- Consistent artistic direction.
- Stable rendering.
- No major visual artifacts.
- Maintainable architecture.
- Documented implementation decisions.

Stop after Phase 7 and wait for approval.

#### Phase 8 — Player

Objective: put a wanderer in the wilderness.

Tasks:

- True 90° player sprite (head/shoulders, no 3/4 face).
- WASD movement with AABB vs water and trunks (X then Y).
- Camera deadzone follow, pixel snap.
- Cover fade: only the canopy over the player.

Stop after Phase 8 and wait for approval.

#### Phase 9 — Audio

Objective: hear the forest. Rain, wind, footsteps. Not this phase.

### 15. Development Rules

These rules apply throughout the project.

1. Complete one phase before starting the next.
2. Start with Phase 1 only.
3. Do not implement future phases prematurely.
4. Keep the project runnable throughout development.
5. Use small, maintainable Rust modules.
6. Avoid unnecessary dependencies.
7. Do not replace existing architecture without a concrete reason.
8. Document important rendering decisions.
9. Test each phase visually, not only through compilation.
10. Do not generate placeholder implementations that pretend to provide effects that are not actually implemented.
11. Respect the artistic direction in every visual decision.
12. Avoid unnecessary abstractions and speculative systems.
13. Prioritize a working implementation over excessive architecture.
14. Do not introduce gameplay systems unrelated to the current phase.
15. Do not continue automatically after completing a phase.
16. If the player is under a covering prop, fade only that prop (see-through / invisibility). Never hide the whole forest.

Phase Completion Report

At the end of each phase, provide:

- Summary of implemented features.
- Important files created or modified.
- Dependencies introduced and why.
- Build and execution results.
- Visual verification results.
- Known limitations.
- Any architectural decisions affecting future phases.

Then stop and wait for explicit approval.

### 16. Final Visual Objective

The final result should resemble a high-definition pixel-art natural world with sophisticated environmental rendering.

The player should feel as though they are exploring a living wilderness.

The terrain and objects should look handcrafted, detailed and cohesive.

Water, sunlight, shadows and atmosphere should provide realism without turning the scene into a photograph or a conventional 3D game.

---

## Arquitectura prevista (Phase 1)

No crear módulos vacíos. Solo lo que Phase 1 necesite:

```
Coquillage/
  plan.md
  Cargo.toml
  src/main.rs
  src/app.rs          # winit ApplicationHandler
  src/renderer.rs     # wgpu: world target + present
  src/camera.rs       # ortho top-down, pan
  shaders/world.wgsl
  shaders/present.wgsl
  assets/reference/
```

Pipeline:

1. **World layer** — render a textura `640×360`, sampler nearest, tiles 32×32 de prueba.
2. **Present layer** — blit nearest-neighbor, escala entera, letterbox. Aquí se enganchará luego la capa ambiental a resolución nativa.
3. Cámara ortográfica en píxeles de mundo. WASD para comprobar el pan.

Gráficos de Phase 1: patrón de prueba honesto (hierba / tierra / sendero generados en código). No fingir lighting, agua ni clima.

---

## Estado

| Item | Estado |
| --- | --- |
| Workspace Coquillage | hecho |
| Repo GitHub renombrado y vaciado | hecho (`main`) |
| Referencia copiada a `assets/reference/` | hecho |
| Este `plan.md` | hecho |
| Pixler MCP | pendiente de token `pxl_live_...` |
| Código Rust Phase 1 | hecho (aprobado al pedir Phase 2) |
| Código Rust Phase 2 | hecho (aprobado al pedir seguir) |
| Código Rust Phase 3 | hecho (aprobado al pedir Phase 4) |
| Código Rust Phase 4 | hecho (aprobado) |
| Código Rust Phase 5 | hecho (aprobado) |
| Código Rust Phase 6 | hecho (aprobado al pedir Phase 7) |
| Código Rust Phase 7 | hecho (aprobado al pedir Phase 8) |
| Código Rust Phase 8 | player (falta OK visual) |
| Phase 9 audio | bloqueada hasta aprobar Phase 8 |

Cuando termine Phase 1, el informe de cierre va en la siguiente sección.

---

## Informe de fases

### Phase 1 — Rendering Foundation

**Estado:** código listo y **compila**. Parado a la espera de tu aprobación visual. No se empezó Phase 2.

**Qué hay:**

- App Rust (`winit` 0.30 + `wgpu` 30), sin Godot / Unity / Bevy.
- Cámara top-down ortográfica, pan con WASD / flechas, Escape sale.
- Mundo lógico 640×360, tiles 32×32, upscale nearest-neighbor a escala entera + letterbox.
- Offscreen world target → present pass (hueco para la capa ambiental).
- Terreno de prueba generado en código: hierba A/B, tierra, piedra; sendero de 3 tiles.

**Cómo probarlo (esta máquina, sin Visual Studio):**

```powershell
$env:Path = "C:\Users\mario\w64devkit\bin;" + $env:Path
cd C:\Users\mario\workplace\Coquillage
cargo +coquillage-gnu run --release
```

Ventana **Coquillage — Phase 1** a 1280×720. Comprueba píxeles nítidos al redimensionar, letterbox si no es múltiplo de 640×360, y que el sendero se vea al mover la cámara.

**Toolchain local:** `rustc` GNU 1.96 (`coquillage-gnu`) + w64devkit gcc. El host MSVC no tiene `link.exe`.

**Pixler:** sigue sin token; Phase 1 usa gráficos de prueba.

### Phase 2 — Terrain and Asset Pipeline

**Estado:** código listo. Parado a la espera de tu aprobación visual. No se empezó Phase 3.

**Qué hay:**

- PNG en `assets/terrain/` (hierba A/B, tierra, piedra, arena 32×32; pebble y tuft 16×16).
- `manifest.json` como catálogo. El renderer solo recibe atlas + instancias.
- Atlas packed en runtime (`src/assets.rs`). Transiciones baked por vecindario 4-dir (`src/world.rs`).
- Mapa: pradera, sendero de tierra/piedra, dos claros de arena.
- Decals de 16×16 encima, misma densidad (1 px de sprite = 1 px de mundo).
- Dependencia nueva: `image` (PNG) y `serde_json` (manifiesto).

**Cómo probarlo:**

```powershell
$env:Path = "C:\Users\mario\w64devkit\bin;" + $env:Path
cd C:\Users\mario\workplace\Coquillage
cargo +coquillage-gnu run --release
```

Ventana **Coquillage — Phase 2**. Recorre con WASD: hierba, sendero, piedra, arena, bordes mezclados, pebbles/tufts nítidos al redimensionar.

**Limitaciones:** transiciones son dither de borde, no autotile blob de 16 variantes. Pixler sigue fuera.

**Arquitectura para fases siguientes:** sprites nuevos = PNG + fila en el manifiesto. El present pass no cambió.

### Phase 3 — Vegetation and Environmental Composition

**Estado:** **aprobada**. Arte Slynyrd + hierba alta/baja + tronco con volumen + sendero que tilea.

**Qué hay:**

- Roble = copa de manchas; pino = pads apilados Slynyrd; abedul más claro. Tocón con raíces.
- Césped de suelo + hierba corta/alta como plantas. Flores en clusters; hongos raros.
- Cover fade apagado hasta que exista el jugador (el centro de cámara ghosteaba la copa y dejaba la sombra).

### Phase 4 — Lighting and Shadows

**Estado:** **aprobada** (siguiente fase).

**Qué hay:**

- Uniform `sun_dir`: hierba alta, helechos, arbustos y árboles proyectan blob morado que se mueve con el sol.
- Present pass: hue-shift cálido/morado, AO direccional barato, shafts de 1px nearest-neighbor.
- Q / E rotan el sol. Sin oscilación automática. Cover fade apagado hasta el jugador.
- Pixel-art intacto: la luz es un multiply a 640×360, luego upscale entero.

**Limitaciones:** no es un shadow map 3D. Shafts son bandas de 1px.

### Phase 5 — Water and Atmospheric Effects

**Estado:** aprobada (siguiente etapa).

**Qué hay:**

- Estanques (profundo / somero) con orilla de arena y labio de espuma, anclados a la grilla 16×16. El del centro está en la cámara inicial.
- Tiles de terreno 32×32; cada uno son 4 celdas interactuables de 16×16. Arena entre agua y tierra.
- Agua animada: overlay con alpha (la tierra no hereda el shimmer). Destellos de sol world-locked.
- Haze frío en el present pass (mix de color, no desenfoque).
- Maximize reconfigura el swapchain al tamaño real y usa la mayor escala entera.

**Cómo probarlo:** ventana **Coquillage — Phase 5**. Busca el estanque teal, espera las ondas. Maximiza: el bosque debe crecer, no quedar un recuadro chico. Los píxeles siguen nítidos.

**Limitaciones:** no hay reflexión de árboles en el agua (no hay buffer de reflexión).

### Phase 6 — Environmental Animation and Weather

**Estado:** **aprobada** (pedido de siguiente fase).

**Qué hay:**

- Viento: hierba, helechos, arbustos y copas se inclinan 1–2 px (más arriba). El suelo no se mueve.
- Lluvia que cae (cabeza abajo, cola arriba), translúcida. Splash y suelo mojado en celdas descubiertas.
- Ciclo día/noche con grade, sombras ovales y Q/E. **R** activa / desactiva lluvia.

**Limitaciones:** no hay charcos persistentes ni física de gotas.

### Phase 7 — Visual Consistency and Optimization

**Estado:** código listo. Parado a la espera de tu OK visual.

**Qué hay:**

- Review de assets: sin sombra horneada donde ya hay blob dinámico; 1 px sprite = 1 px mundo.
- Atlas: padding 1 px + extrusión de borde (anti-bleed, nearest, sin mips).
- GPU: 2 draw calls. Solo se suben las instancias de sombra cada frame. Uniform de cámara si hubo pan.
- Título con FPS una vez por segundo. Decisiones en `DEV.md` §18.

**Cómo probarlo:** ventana **Coquillage — Phase 7**. Recorre el bosque: nada de hierba encima de copas, sombras de un solo blob, título ~60 fps. Q/E y R siguen igual.

### Phase 8 — Player

**Estado:** código listo. Parado a la espera de tu OK visual.

**Qué hay:**

- Figura 16×16 a 90° (capa GOLD, 4 direcciones, 2 frames de paso).
- WASD mueve al jugador. Hitbox 10×7. Agua y troncos sólidos; hierba y arbustos no.
- Cámara con deadzone 80×48, lerp y snap a píxel.
- Si estás bajo una copa, **solo esa** se pone ghost (~0.32).

**Cómo probarlo:** ventana **Coquillage — Phase 8**. Camina al agua (para), métete bajo un árbol (la copa se transparenta), pasa delante (tú encima). Q/E y R igual.
