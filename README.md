<p align="center">
  <img src="assets/brand/coquillage-logo.png" alt="Coquillage" width="520">
</p>

<p align="center">
  <img alt="Rust 1.96" src="https://img.shields.io/badge/rust-1.96-c45c26?style=flat-square">
  <img alt="wgpu 30" src="https://img.shields.io/badge/wgpu-30-5b4b8a?style=flat-square">
  <img alt="winit" src="https://img.shields.io/badge/winit-0.30-3d6b4f?style=flat-square">
  <img alt="top-down" src="https://img.shields.io/badge/cámara-90°%20top--down-d6c24a?style=flat-square">
  <img alt="640x360" src="https://img.shields.io/badge/lógico-640×360-7aa2b0?style=flat-square">
</p>

<p align="center">
  Bosque a vista de pájaro. Render propio. Sin motor.
</p>

Caminas un claro: agua, troncos, copas que se abren solo cuando pasas debajo. El sol mueve la luz de las hojas. De noche entra la luna. Una llama cerca del claro tira a naranja.

## Jugar

Rust 1.96 y un linker de C. En Windows, MinGW (w64devkit) en el `PATH`, y el toolchain gnu que ya usa esta máquina.

```powershell
cargo run --release
```

La ventana se llama **Coquillage — Phase 8**, a 1280×720. El título muestra los FPS.

| Tecla | Hace |
| --- | --- |
| W A S D o flechas | Caminar |
| Q / E | Saltar una hora |
| R | Lluvia |
| Escape | Salir |

## Qué mirar

- Figura chica desde arriba. Cabeza arena, camisa azul, manos de cuatro píxeles.
- El agua y los troncos frenan. La hierba no.
- Bajo una copa, solo esa copa se transparenta.
- Q y E mueven la mancha clara de las hojas. La llama del claro no cambia de color con la hora.

Los sprites salen de `tools/gen_props.py` y `tools/gen_tiles.py`.
