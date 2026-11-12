# Coquillage — DEV.md (guías para agentes)

Objetivo: **el mejor videojuego posible**. Este archivo es la biblia operativa. Léelo antes de tocar arte, shaders, terreno o pipeline.

`plan.md` = qué fase y qué decidió Mario.  
`assets/reference/pixel-art-bible.md` = paleta y anti-slop.  
Este archivo = **cómo** hacerlo bien, con guías de gamedev. No inventes.

---

## 0. Orden de autoridad

1. Lo que Mario diga en el chat (gana siempre).
2. Refs visuales aprobadas en `assets/reference/` (no inventar estilo).
3. Este `DEV.md` + `pixel-art-bible.md`.
4. Guías de gamedev buscadas **en esta sesión** (enlaces abajo + búsqueda nueva).
5. Tu intuición: último recurso, y dilo.

Si Mario no especificó el look o la técnica **y no estás seguro: busca**. Visual incluido. No “inventar un efecto bonito”.

Parar al terminar una fase. No Phase N+1 sin OK visual. No commit/push salvo que lo pida. No capas subterráneas hasta que las pida.

---

## 1. Protocolo de investigación (obligatorio)

Antes de implementar o retocar algo visual/técnico que no esté cerrado:

1. Nombra el problema en una frase (“lluvia que se lea cayendo”, “sombra blob vs sol”).
2. Busca 2–3 fuentes de **gamedev / pixel art / rendering**, no blogs genéricos de IA.
3. Extrae 3 reglas concretas (dirección del motion, ancla, paleta, qué no hacer).
4. Encájalas en Coquillage: top-down ortográfico estricto, 1 sprite-px = 1 world-px, paleta `tools/pix.py`, nearest-neighbor.
5. Si choca con una ref o con Mario, gana Mario/ref.

Consultas útiles: `pixel art <tema>`, `2D <tema> shader`, `top-down <tema>`, `Saint11`, `Slynyrd Pixelblog`, `Lospec`, `gamedev`.

---

## 2. Hechos del proyecto (no reinventar)

| Cosa | Valor |
| --- | --- |
| Motor | Rust + wgpu 30 + winit 0.30. Sin Godot/Unity/Bevy |
| Cámara | Top-down **ortográfica estricta**. Nunca isométrica ni 3/4 |
| Lógico | 640×360. Tile visual 32×32. Occupancy 16×16 |
| Atlas | 1 px sprite = 1 px mundo. Sampler nearest |
| Present | Offscreen world → blit. Integer scale; fill si el área entera < 78% |
| Paleta | `tools/pix.py`: `LEAF_HI/MID/DK/SH`, `GOLD`, `WATER_*`, `FOAM` |
| Sombras | Moradas (`LEAF_SH`). Luces amarillo-lima / `GOLD` |
| Árboles | Pads de follaje apilados (Slynyrd 44). **Prohibido** rueda de radios |
| Capas | `ground` / `floor` / `cover` en celdas 16×16. CPU. Sin subterráneo |
| Cover de árbol | Solo tronco (celda de ocupación en la base). La copa no tapa lluvia |
| Fade de copa | Solo el instance sobre el jugador. **Off** hasta que exista jugador |
| Toolchain | `cargo +coquillage-gnu`, Path + `C:\Users\mario\w64devkit\bin` |
| Exe | `target\x86_64-pc-windows-gnu\release\coquillage.exe` |

Refs que mandan (en este orden):

1. `assets/reference/minecraft-hd-lighting.jpg`
2. `assets/reference/wilderness-lighting-sprites.jpg`
3. `assets/reference/slynyrd-conifer-topdown.png`
4. Construcción (no cámara): `slynyrd-isometric-tree.png`
5. Evitar: `current-look-avoid.png`, `polka-shadows-avoid.png`

Ambiente: solo naturaleza. Sin ciudades, NPCs, armas.

---

## 3. Cámara y perspectiva

**Coquillage:** ortho Y-down, pan WASD, sin convergencia, sin isometría.

**Guías:**

- [OpenGameArt — Perspectives](https://opengameart.org/content/chapter-3-perspectives) — axonometría vs isométrica. Coquillage **no** es isométrica.
- [LPC Style Guide — Perspective](https://lpc.opengameart.org/static/LPC-Style-Guide/build/styleguide.html) — ortográfico, las cosas no encogen con la distancia. LPC usa ~60°; **nosotros 90° puro**.
- [RPG in a Box — 2D orthogonal 90°](https://rpginabox.com/docs/doku.php?id=2d_games) — pitch 90, ortho, sin rotar cámara.
- [Sandro Maglione — top-down sprites](https://www.sandromaglione.com/articles/pixel-art-top-down-game-sprite-design-and-animation) — sombras de contacto para altura; el sprite se lee desde arriba.

**Hacer:** todo se dibuja como visto desde arriba. Tronco = disco/elipse + corteza, no alzado.  
**No:** copiar Slynyrd 160 / sheet 3/4 como cámara. Esos son método de copa, no ángulo.

---

## 4. Pixel art, paleta, anti-slop

**Coquillage:** clusters, hue-shift, 4–6 colores por material, silueta clara a 1×.

**Guías:**

- [Saint11 — Basic Shading](https://saint11.art/pixel_art_articles/article4/)
- [Saint11 — Color Theory / hue shift](https://saint11.art/pixel_art_articles/article6/)
- [the-pixel.art — Hue shifting](https://the-pixel.art/articles/hue-shifting-pixel-art-shading/) (~15–25°; luces a amarillo, sombras a morado/azul; saturación máxima en el medio)
- [Pixeli — Clusters and noise](https://pixeli.pebbly.space/knowledge/rules/11-clusters-and-noise)
- [Pixeli — Shading and light](https://pixeli.pebbly.space/knowledge/rules/02-shading-and-light)
- [pixelart.school — pillow / banding / strays](https://pixelart.school/learn/pixel-art-terms)
- [How to spot AI pixel art](https://www.spriteforge.tech/blog/how-to-spot-ai-pixel-art)
- Paletas: [Essence24](https://lospec.com/palette-list/essence24), [Purple Sunset](https://lospec.com/palette-list/purple-sunset)

**Hacer:** una sola luz; sombra de volumen a un lado, no anillo. Detalle en clusters de 3–8 px y aire.  
**No:** sombrear a gris, pillow shading, píxel huérfano, dither de sal, confeti, simetría radial, borde de tile recuadrado.

Los shaders de grade **mezclan hacia** `LEAF_HI` / `GOLD` / `LEAF_SH` / `WATER_SH`. No inventar naranja.

---

## 5. Resolución y present

**Coquillage:** world 640×360 nearest → present. Integer scale; si el rect entero cubre < 78% de la ventana, fill. Píxeles de mundo enteros.

**Guías:**

- [Saint11 — Consistency / scaling](https://saint11.art/blog/consistency/) — canvas que escale entero; nearest; letterbox.
- [Flukz — Resolution scaling](https://flukz.org/devlog/resolution-scaling-shmup/) — 640×360 encaja 1080p@3×. Pipeline: offscreen lógico + blit. UI nativa aparte (cuando exista).
- [Construct — Letterbox integer scale](https://www.construct.net/en/blogs/construct-official-blog-1/feature-focus-display-scaling-1592)
- [Pixel-perfect rendering](https://gamedevaitools.com/concepts/pixel-perfect/) — snap a enteros; padding en atlas contra bleed.

**Hacer:** 1 sprite-px = 1 world-px. No rotar/escalar sprites a factores rotos.  
**No:** bilinear sobre el atlas. No “HD smooth”. Efectos de clima van en present (pueden ser 1 px lógicos, no fotoreal 3D).

---

## 6. Terreno y anti-tiling

**Coquillage:** tiles 32×32; césped = tile verde; hierba = mechones encima. Arena irregular entre agua y tierra. Occupancy 16×16 invisible.

**Guías:**

- [Slynyrd Pixelblog 20 — Top Down Tiles](https://www.slynyrd.com/blog/2019/9/9/pixelblog-20-top-down-tiles)
- [Cobo — grass/paths that don't look tiled](https://cobomakesgames.com/blog/pixel-art-grass-and-paths/) — forma (tile) vs color (ruido world-locked); bordes en clusters 2×2, no checkerboard.
- [Neta — collision layer ≠ visual tileset](https://neta.art/blog/tiled-map-workflow-tileset-to-playable-level)
- [LPC](https://lpc.opengameart.org/static/LPC-Style-Guide/build/styleguide.html) — grid 32, subtiles 16.

**Hacer:** transiciones orgánicas (ruido, no anillo perfecto). Variantes A/B. World-lock.  
**No:** cambiar el tamaño visual del tile a 16. No grid naranja. No agua que mueva la tierra.

---

## 7. Árboles y vegetación

**Coquillage:** `slynyrd_pad` en `tools/pix.py`. tree_a 96×96, tree_b 64×96, tree_c 64×64. Arbustos/helechos 32. Hierba alta 24×32, corta/flores 16.

**Guías:**

- [Slynyrd 44 — Top Down Trees](https://www.slynyrd.com/blog/2023/5/22/pixelblog-44-top-down-trees) — un bundle de hojas × 3 valores (hi/mid/dk) apilados; tronco; sombra mancha.
- [Slynyrd 2 — Modular foliage](https://www.slynyrd.com/blog/2018/2/15/pixelblog-2-texture)
- [Slynyrd 21 — Top Down Objects](https://www.slynyrd.com/blog/2019/9/18/pixelblog-21-top-down-objects) — formas simples para repetir; asimétricos solo como acento.

**Hacer:** pads / manchas, tronco entre capas, copa irregular tipo brócoli. Variantes, no filas.  
**No:** estrella, radios, elipse lisa, hongos a intervalos iguales.

Césped ≠ hierba. La hierba se mece; el césped no.

---

## 8. Capas, occupancy, cover

**Coquillage:** `Layers { ground, floor, cover }` en `src/world.rs`. Celda 16×16.  
`ground` = material + `G_TAKEN`. `floor` = pebble/flor/hierba corta. `cover` = lo que tapa el suelo (arbustos = occupancy; árboles = **solo tronco** en la base del sprite).  
Lluvia se ve en todo. Moja/splash solo si `cover==0` y no es agua. Subterráneo: **no ahora**.

**Guías:**

- [Collision AABB vs tiles](https://gamedev-canon.alekseiprikhodko.com/m01-collision.html) — occupancy independiente del sprite.
- [Gauntlet collision / cbox](https://jakesgordon.com/writing/javascript-gauntlet-collision-detection/) — hitbox ≠ tamaño visual.
- [Sprite bigger than tile](https://gamedev.stackexchange.com/questions/70276/collision-detection-via-adjacent-tiles-sprite-too-big) — un árbol cubre N celdas; testea el rect, no 1 celda.

**Hacer:** 1 objeto interactivo por celda de occupancy. Copa visual puede colgar fuera.  
**No:** marcar cover con el AABB entero de la copa (eso apagaba la lluvia sobre los árboles).

### Fade bajo copa (cuando exista jugador)

Solo **ese** instance, alfa ~0.3, sin blur. Probe = posición del jugador, no el centro de cámara.

- [GameDev.tv — fade only nearby canopy tiles](https://community.gamedev.tv/t/transparency-of-canopy-based-on-player-position/236352)
- [Godot — modulate the one tree, not all](https://forum.godotengine.org/t/make-a-wall-tree-a-little-bit-translucent-trasnparent-when-there-is-a-player-behind-it-in-2d-top-down-especially/111322)

---

## 9. Iluminación y ciclo día/noche

**Coquillage:** 24 min = 24 h. Q/E salta hora. Half-Lambert + Sobel normals. Grade en present hacia la paleta. Sol/luna disco en present.

**Guías:**

- [Half-Lambert (Valve)](https://github.com/DeGGeD/ShaderStory/blob/main/Chapters/Lighting/Models/HalfLambertian.md) — `wrapped = saturate((NdotL+wrap)/(1+wrap))`; silueta legible en sombra.
- [Sprite Lamp wrap lighting](https://www.snakehillgames.com/sprite-lamps-basic-shader/)
- [2D day/night via LUT / tint](https://www.reddit.com/r/gamedev/comments/7cti1q/2d_daynight_cycles/) — LUT o tints; conservar emisivos.
- [Joost — recolour daylight art for night](http://joostdevblog.blogspot.com/2014/11/using-2d-daylight-assets-to-create.html) — un multiplicador global, luego excepciones.

**Hacer:** un sol. Atardecer = paleta (GOLD + LEAF_SH), no filtro naranja. Niebla baja no debe volver el sunset gris.  
**No:** shafts molestos; lattice naranja; pulir hasta fotoreal 3D.

La mancha clara de una copa **no va horneada** en el albedo. El pad es volumen radial (medio / oscuro / morado en el borde). El cluster brillante lo pinta el shader en los píxeles cuya normal mira a la luz, y se mueve cuando cambia el azimut.

Reglas de esta sesión:

- [Sprite Lamp](https://www.snakehillgames.com/spritelamp/) — el difuso es la forma con luz pareja. La dirección vive en el normal map (aquí, Sobel del volumen del pad, no de un highlight pintado).
- [Sprite Lamp wrap](https://www.snakehillgames.com/sprite-lamps-basic-shader/) — `clamp(dot(n, L) + wrap, 0, wrap + 1) / (wrap + 1)`. Wrap ~0.5 para que la silueta no muera.
- [Pixeli — lighting scenarios](https://pixeli.pebbly.space/knowledge/rules/24-lighting-scenarios) — una key manda. Sol: highlight hacia amarillo (`LEAF_HI`), sombra hacia violeta (`LEAF_SH`). Luna: highlight pálido frío (`WATER_HI`), más débil. Antorcha: rampa de fuego, naranja `(232, 120, 48)` y núcleo amarillo pálido `(255, 199, 107)`, caída al cuadrado. El naranja es solo de la llama, no del atardecer.
- [the-pixel.art — hue shift](https://the-pixel.art/articles/hue-shifting-pixel-art-shading/) — el sentido del hue lo dicta la luz. Sol y fuego empujan el claro al amarillo. La luna lo empuja al cian, no al dorado.

Dos luces puntuales (`point0`, `point1`) junto a copas grandes. Radio ~88 px. Sirven para antorchas; no hay sprite de antorcha todavía.

---

## 10. Sombras dinámicas

**Coquillage:** blob `shadow` / `shadow_sm`. Sol alto → casi círculo. Sol bajo → óvalo largo. Ancla en la base. Purple.

**Guías:**

- [Unity Happy Harvest — blob shadows stretch/rotate with sun](https://discussions.unity.com/t/2d-light-and-shadow-techniques-with-the-universal-render-pipeline/1641848)
- [Construct — ellipse shadow, origin at contact, length from elevation](https://www.construct.net/en/tutorials/cast-shadows-change-time-494/page-3)
- [VectorStorm — blob shadows](https://www.vectorstorm.com.au/2012/05/15/lets-talk-about-shadows/)
- [Sandro Maglione — contact shadows = altura](https://www.sandromaglione.com/articles/pixel-art-top-down-game-sprite-design-and-animation)

**Hacer:** mancha, no proyección hard-shadow 3D. Capar longitud al atardecer.  
**No:** lunares morados regulares (`polka-shadows-avoid.png`). No sombra horneada en el sprite del árbol.

---

## 11. Agua

**Coquillage:** shallow/deep, foam que avanza/recde **dentro** de sí (no come tierra), sparkles world-locked, flags agua `0.5–1.5`.

**Guías:**

- [Pixeli — Water](https://pixeli.pebbly.space/knowledge/rules/64-water) — stack: arena → wet → foam 1–2 px irregular → shallow claro → mid → deep. Foam en diagonal, no pulso uniforme.
- [Cainos — top-down water](https://docs.cainos.net/pixel-art-top-down-village/water-rendering) — foam desplazada 1 px, caústicos stepped.
- [Shoreline foam surge/retreat](https://draw.love-ink-llc.com/animatefx/BCAFXContent/BCAFX_Level-12/1204_48_Shoreline.html)

**Hacer:** orilla irregular (1 celda de arena, 2ª ruidosa). Profundidad en la misma superficie.  
**No:** franjas que se mueven como conveyor. No anillo de arena que encierra césped.

---

## 12. Viento

**Coquillage:** flags planta `2.1–4.0`. Sway 1–2 px, más en copa. Pivot en la base. World-space para que el bosque no baile al unísono.

**Guías:**

- [pixelgraphics foliage/grass shaders](https://github.com/aarthificial/pixelgraphics/blob/master/Documentation~/shaders.md) — hierba vs arbusto; pixelart = distorsión UV, mesh un poco mayor.
- [2D tree wind — noise from world pos, pivot bottom](https://whwogur.vercel.app/Shader/2D%20Tree%20Wind%20and%20Hit%20Effect/)
- [GDSE sine wind](https://gamedev.stackexchange.com/questions/74100/how-do-i-make-a-sprite-appear-to-sway-with-the-wind) — offset por `sin(time + pos)`, más en la punta.
- [Godot simple wind 2D](https://github.com/JaccomoLorenz/godot-simple-wind-shader-2d) — `heightOffset` para no mover la raíz.

**Hacer:** sutil. Con lluvia un poco más. Enteros de píxel.  
**No:** 4+ px, ni que se lea como jelly.

---

## 13. Lluvia

**Coquillage:** present.wgsl. Cae **en todo**. World-locked. Translúcida y chica. Splash/wet solo suelo descubierto (máscara R8). **R** toggle.

Y-down: **restar** tiempo a Y. Cabeza brillante **abajo**, cola arriba. Si sumas tiempo o pones el brillo arriba, **sube**.

**Guías:**

- [t3ssel8r — Pixel Art Rain](https://www.gamesinprogress.com/indie-game-developers/t3ssel8r/pixel-art-rain-shader) — capas: streaks + splash suelo + spatter techos + ripples agua + mist + bob de hojas + grade. GPU, no mil partículas CPU.
- [Keyframer — fract trails](https://keyframer.dev/recipes/matrix-rain-shader-react-native-skia) — signo del time según si Y crece abajo.
- [Baltor — 3 capas de streaks](https://baltor.ai/gallery/rain-streaks-2d) — far/mid/near, distinta velocidad/opacidad.
- [Sébastien Lagarde — rain effects](https://seblagarde.wordpress.com/2012/12/27/water-drop-2a-dynamic-rain-and-its-effects/) — streaks y splashes **sistemas independientes**; splash solo cerca de cámara / en geometría vista desde arriba.
- [Godot rain-drop floor ripples](https://godotshaders.com/shader/rain-drop-floor-effect/) — world-space, no screen-space que nade con la cámara.
- [Pixel raindrop splash anatomy](https://particlefx.studio/effects/pixel-raindrop-splash) — raya → corona → ripple. En Coquillage: anillo 2–3 px, no cruz blanca.

**Hacer:** casi vertical (slant ≤ ~0.08). Tres velocidades. Mix bajo (~0.28).  
**No:** diagonal 0.42 (parece que vuela). No cruces opacas sobre las copas.

Wet: oscurecer + sheen frío solo si `sky_open`. Agua ya está mojada: no sheen extra.

---

## 14. Atmósfera (niebla, shafts, aura)

**Coquillage:** haze en `LightUniform`, shafts discretos. El aura es solo el borde.

De día, hasta ~15:30, el borde está apagado. Empieza a notarse camino a la noche (`smoothstep` 15.5→19.5) y de madrugada se apaga otra vez hacia el alba. El seco nunca pasa de ~0.07 en la esquina. Con lluvia el borde es más ancho y llega a ~0.13, y el seco no lo alcanza. No hay un segundo oscurecido fijo en las esquinas.

**Guías:**

- [Pixelated God Rays](https://godotshaders.com/shader/pixelated-god-rays/) — cuantizar, no volumétrico suave 3D.
- [Animated 2D fog](https://godotshaders.com/shader/procedural-2d-fog-with-pixelation/)
- Minecraft-HD ref: marco oscuro de dosel = viñeta natural, no CRT horror.

**Hacer:** UN POCO. Bonito, no molesto. Combinar con paleta. El reloj va bajo el sol: placa `LEAF_SH` y dígitos `FOAM` a 2×, porque un 3×5 del color del disco se pierde en la copa.  
**No:** viñeta negra de día, aura de atardecer tan fuerte como la de noche, aura seca más fuerte que la de lluvia, shafts de discoteca, fog que lava el atardecer.

---

## 15. Pipeline wgpu 2D

**Coquillage:** instance quads + atlas + normals. World pass → present pass. Uniforms bytemuck.

**Guías:**

- [SpriteBatch render textures](https://spritebatch.com/guides/render-textures/) — offscreen + point sampling.
- [Flukz two-layer pipeline](https://flukz.org/devlog/resolution-scaling-shmup/)
- wgpu: instancing, `R8Unorm` para máscaras (pad width a 256), `textureLoad` sin filtro.

Atlas: 1 px de padding + **extrusión del borde** (no gutter transparente). `textureLoad` nearest, sin mips. `size_xy` del instance = tamaño del sprite, no “roca+arbusto” estirado.

Cada frame: 2 draw calls (world instanced + present fullscreen). Solo se resuben las instancias de sombra; el resto del instance buffer vive en GPU. Cámara se escribe si se movió.

---

## 16. Jugador (Phase 8)

**Coquillage:** sprite 32×32 visto a 90° (cabeza chica de arena, camisa `SHIRT` azul, no `GOLD`). Manos 2×2 planas con una esquina oscura, al lado de las costillas, no elipses. Hitbox 10×8 en los pies. WASD mueve al jugador. Cámara con deadzone + lerp + snap. Fade de copa: un solo instance, probe = jugador.

Reloj: fuente 3×5 bajo el sol/luna, HH:MM del `day_hour`, 1 px de sombra (no outline). Subtle, paleta disc + FOAM.

**Guías:**

- [Pixeli — hands](https://pixeli.pebbly.space/knowledge/rules/34-hands-and-feet) — bajo 16 px la mano no es un disco. Un borde plano y una esquina oscura. El pulgar no se dibuja en una mano relajada.
- [Nikou — top-down hands](https://www.nikouusitalo.com/blog/looking-at-pixels-part-1-characters/) — a este tamaño la mano es 2×2, más chica que el torso. Un par de píxeles basta para el paso.
- [Framesprite — true 90° character](https://www.framesprite.com/guides/top-down-character-sprite-generator) — cara oculta; cabeza, hombros y huella mandan.
- [Pixeli — views](https://pixeli.pebbly.space/knowledge/rules/37-views-and-directions) — N = pelo/nuca; W = flip de E; 4 direcciones.
- [AABB vs tiles, eje X luego Y](https://gamedev-canon.alekseiprikhodko.com/m01-collision.html) — evita enganchar esquinas.
- [GDCS camera deadzone](https://gdcs.coody.app/game-feel/camera/camera-deadzone) — caja, luego `1-e^(-k dt)`, snap al pintar.
- Fade: solo ese árbol/arbusto, alfa ~0.32. Hitbox de fade más chica que el sprite.

**Hacer:** colisión con agua y troncos. Hierba/flores/arbustos se cruzan. Y-sort por pie con la cover.  
**No:** personaje 3/4 con cara. No pan libre de cámara. No fade de todo el bosque.

---

## 16b. Audio / Phase 9

No implementar hasta la fase. Cuando toque: buscar guía **en esa sesión** (footsteps mojados, wind loop) y mover esta nota a una sección propia.

---

## 17. Decisiones de render (Phase 7)

- **World 640×360** offscreen, present integer/fill. Efectos de clima en present, geometría pixel en world.
- **Un atlas** albedo + normals Sobel. Instantes: `world_xy, atlas_xy, size_xy, atlas_wh, alpha, flags`.
- **Capas de dibujo:** terreno → agua → decals → sombras blob → floor → cover (altura, luego pie Y).
- **Sombras:** sprites dinámicos, no horneadas en el prop. Alpha = `shadow_str` del ciclo.
- **Luz:** Half-Lambert en world; grade/haze/aura/lluvia en present. Paleta `tools/pix.py`.
- **Occupancy 16×16** ≠ sprite visual. Un objeto por celda; copa puede colgar.
- **Sin Bevy/Godot.** wgpu 30, winit 0.30, gnu toolchain.

---

## 18. Checklist antes de decir “listo”

- [ ] ¿Mario lo pidió en **esta** fase?
- [ ] ¿Hay guía (este doc o búsqueda nueva), no paleta inventada?
- [ ] ¿Sigue top-down puro y paleta `pix.py`?
- [ ] ¿Compila con `cargo +coquillage-gnu --release` y se relanzó el exe?
- [ ] ¿Se vio en pantalla el flujo, no solo un frame mental?
- [ ] ¿Se paró para OK visual?

Efectos: hermosos y discretos. Si molesta, baja intensidad antes de “añadir más”.
