# Zaralle — pixel art, color, anti-slop

Primary look (in this order):

1. [`minecraft-hd-lighting.jpg`](minecraft-hd-lighting.jpg) — sol cálido, hojas en **manchas**, corteza leíble, sombras frías.
2. [`wilderness-lighting-sprites.jpg`](wilderness-lighting-sprites.jpg) — bosque HD, sendero, dosel oscuro.
3. [`slynyrd-conifer-topdown.png`](slynyrd-conifer-topdown.png) — **método** de pino top-down: capas de follaje, no rueda de radios.
4. [`slynyrd-isometric-tree.png`](slynyrd-isometric-tree.png) — construcción (Slynyrd 160): disco plano → anillos hue-shift (lima / verde / morado) → silueta de brócoli → tocón + raíces. **No copiar la cámara isométrica.** Desde arriba: copa redonda irregular, rampa NW→SE, log + nudos de raíz.

Apoyo (forma de copa, no cámara): [`topdown-map-canopies.png`](topdown-map-canopies.png). El sheet 3/4 [`tree-sheet-34-view.png`](tree-sheet-34-view.png) solo para densidad de copa; **no** copiar alzado.

Qué no repetir: [`current-look-avoid.png`](current-look-avoid.png) — árboles-estrella, hongos de confeti, rejilla naranja entre tiles.

## Color (hue shift)

Fuentes: [Saint11 — Basic Color Theory](https://saint11.art/pixel_art_articles/article6/), [Hue shifting](https://the-pixel.art/articles/hue-shifting-pixel-art-shading/), [Pixor](https://pixor.art/learn/hue-shifting/), paletas Lospec [Essence24](https://lospec.com/palette-list/essence24) y [Purple Sunset](https://lospec.com/palette-list/purple-sunset).

- La sombra **no** es el mismo verde más oscuro. Gira el hue ~15–25° hacia **morado / azul**.
- La luz gira hacia **amarillo / verde-lima** (el sol de la foto Minecraft).
- Saturación más alta en el medio de la rampa, no en el highlight.
- Un solo sol para todo: luz arriba-izquierda, sombra abajo-derecha.
- Morado en sombras de hojas, tierra, piedra y en la mancha bajo el árbol.

Rampa de hojas (ejemplo):

| rol | RGB |
| --- | --- |
| highlight sol | `200, 214, 86` |
| mid | `62, 122, 48` |
| core | `32, 72, 40` |
| sombra morada | `48, 36, 72` |

## Escala 32×32

- Tile de suelo **32×32**. Props en **16 / 32 / 64 / 96**. Un píxel de sprite = un píxel de mundo.
- El verde del suelo es el **mismo** mid de las copas (`LEAF_MID` en `tools/pix.py`).
- Clusters de hoja a escala de tile (radio 8–16 px, 4 colores). Prohibido apilar elipses suaves.

## Manuales / método

- Slynyrd pino top-down = **módulos de follaje apilados** (pads), tronco entre capas, sombra como mancha, no como blur.
- Slynyrd 160 árbol: volumen = 3 discos hue-shift; la copa se **rompe** en clusters de hoja (no elipse lisa); tocón con raíces; sombra de contacto en el suelo.
- Pixel Logic / ramps: pocas formas grandes, 4–6 colores por material.
- Saint11 shading: dirección de luz constante; evitar pillow shading (anillo oscuro alrededor de todo).

## Anti AI-slop (pixel art)

- Prohibido árbol en **estrella / rueda de radios**.
- Prohibido **píxel suelto** (lonely pixel) y dither de sal.
- Prohibido **confeti**: hongos y flores a intervalos iguales.
- Prohibido **simetría radial** y copias en fila.
- Prohibido sombrear a gris. Hue shift o nada.
- Detalle en **clusters** de 3–8 px, luego aire.
- Silueta clara a 1×. Si no se lee la copa a 64 px, está mal.
- No recuadrar cada tile con un borde de otra textura.
- No ruido por píxel en hierba/tierra/piedra.

Shaders: el grade del present (sombras moradas, luces cálidas, viñeta de dosel) imita la foto Minecraft **sin** fotorealismo 3D. El texel sigue nearest-neighbor.
