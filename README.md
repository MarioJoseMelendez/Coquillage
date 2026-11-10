# Zaralle

Top-down wilderness sandbox. Custom Rust renderer (`wgpu` + `winit`), no game engine.

Phase 1 is a pixel-art terrain test: 640×360 logical resolution, 32×32 tiles, nearest-neighbor integer upscale.

## How to run (Phase 1)

Needs [Rust](https://rustup.rs/) (this machine has 1.96).

```powershell
cd C:\Users\mario\workplace\Zaralle
cargo run --release
```

A window titled **Zaralle — Phase 1** should open at 1280×720 (2× scale).

| Input | Action |
| --- | --- |
| W A S D or arrows | Pan the camera |
| Escape | Quit |
| Resize the window | Integer nearest-neighbor scale + letterbox |

What you should see:

- Top-down grass with a winding cobblestone path and dirt edges
- Sharp pixels, no blur, even after resizing
- Dark letterbox bars if the window is not a multiple of 640×360

Stop after checking that. Phase 2 is not in yet.
