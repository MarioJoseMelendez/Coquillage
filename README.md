# Zaralle

Top-down wilderness sandbox. Custom Rust renderer (`wgpu` + `winit`), no game engine.

Phase 2 loads terrain PNGs from `assets/terrain/`, packs an atlas, and draws grass, dirt, stone and sand with edge transitions. Logical resolution is still 640×360, tiles 32×32, nearest-neighbor integer upscale. 16×16 decals use the same pixel density.

## How to run (Phase 2)

Needs [Rust](https://rustup.rs/) 1.96 and a C linker.

This PC has no Visual Studio `link.exe`. Use MinGW (w64devkit) plus the `zaralle-gnu` rustup toolchain:

```powershell
$env:Path = "C:\Users\mario\w64devkit\bin;" + $env:Path
cd C:\Users\mario\workplace\Zaralle
cargo +zaralle-gnu run --release
```

If you have MSVC Build Tools instead:

```powershell
cd C:\Users\mario\workplace\Zaralle
cargo run --release
```

A window titled **Zaralle — Phase 2** should open at 1280×720 (2× scale).

| Input | Action |
| --- | --- |
| W A S D or arrows | Pan the camera |
| Escape | Quit |
| Resize the window | Integer nearest-neighbor scale + letterbox |

What you should see:

- Grass meadow, winding dirt/stone path, two sand clearings
- Soft dithered edges where materials meet
- 16×16 pebbles and grass tufts that stay sharp when resized
- Dark letterbox bars if the window is not a multiple of 640×360

Add a terrain PNG by dropping it in `assets/terrain/` and listing it in `manifest.json`. Stop after checking this. Phase 3 is not in yet.

To regenerate the built-in tiles:

```powershell
python tools/gen_tiles.py
```
