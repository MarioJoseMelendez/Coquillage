# Zaralle

Top-down wilderness sandbox. Custom Rust renderer (`wgpu` + `winit`), no game engine.

Phase 8 is the player: walk the forest, collide with water and trunks, ghost the canopy you are under. Logical resolution 640×360.

## How to run (Phase 8)

Needs [Rust](https://rustup.rs/) 1.96 and a C linker.

This PC has no Visual Studio `link.exe`. Use MinGW (w64devkit) plus the `zaralle-gnu` rustup toolchain:

```powershell
$env:Path = "C:\Users\mario\w64devkit\bin;" + $env:Path
cd C:\Users\mario\workplace\Zaralle
cargo +zaralle-gnu run --release
```

A window titled **Zaralle — Phase 8** should open at 1280×720 (2× scale). The title shows FPS once per second.

| Input | Action |
| --- | --- |
| W A S D or arrows | Walk |
| Q / E | Skip 1 hour |
| R | Toggle rain |
| Escape | Quit |

What you should see:

- A small cloaked figure, viewed from above (head and shoulders, no face)
- You stop at water and tree trunks; grass and bushes you can cross
- Walking under a canopy fades **that** tree only
- Camera follows with a little slack, still pixel-snapped

Stop after checking that. Audio is Phase 9.

```powershell
python tools/gen_tiles.py
python tools/gen_props.py
```
