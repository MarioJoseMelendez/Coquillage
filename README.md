# Zaralle

Top-down wilderness sandbox. Custom Rust renderer (`wgpu` + `winit`), no game engine.

Phase 3 is a forest floor: terrain PNGs plus top-down tree canopies, bushes, ferns, flowers and mushrooms. Logical resolution 640×360, nearest-neighbor integer upscale. 1 sprite pixel = 1 world pixel.

## How to run (Phase 3)

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

A window titled **Zaralle — Phase 3** should open at 1280×720 (2× scale).

| Input | Action |
| --- | --- |
| W A S D or arrows | Pan the camera |
| Escape | Quit |
| Resize the window | Integer nearest-neighbor scale + letterbox |

What you should see:

- Winding stone path with forest crowding the sides
- Irregular tree canopies (top-down, not side-on)
- Bushes, ferns, flowers; sand clearings more open
- Sharp pixels after resize

Stop after checking that. Phase 4 (lighting) is not in yet.

```powershell
python tools/gen_tiles.py
python tools/gen_props.py
```
