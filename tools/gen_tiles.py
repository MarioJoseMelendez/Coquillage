"""32×32 tiles in the same hue-shift ramp as the Slynyrd pads. No per-tile shadow stamps."""

from __future__ import annotations

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import pix as p

ROOT = pathlib.Path(__file__).resolve().parents[1] / "assets" / "terrain"


def mottled(buf, w, h, salt, mid, dk, hi, hi_chance: float = 0.03) -> None:
    """Fine hue-shift speckle so a 32px tile doesn't stamp stars/blobs when repeated."""
    p.fill(buf, w, h, mid)
    for y in range(h):
        for x in range(w):
            n = p.hash01(x, y, salt)
            if n < 0.10:
                p.put(buf, w, x, y, dk)
            elif hi_chance > 0.0 and n > 1.0 - hi_chance:
                p.put(buf, w, x, y, hi)


def grass(salt: int) -> bytes:
    buf = p.canvas(32, 32)
    mottled(buf, 32, 32, salt, p.LEAF_MID, p.LEAF_DK, p.LEAF_HI, hi_chance=0.0)
    return bytes(buf)


def dirt_tile() -> bytes:
    buf = p.canvas(32, 32)
    mottled(buf, 32, 32, 21, p.DIRT, p.DIRT_DK, p.DIRT_HI)
    return bytes(buf)


def in_cobble(x: int, y: int, cx: float, cy: float, rx: float, ry: float, salt: int) -> bool:
    dx = abs(x + 0.5 - cx) / max(rx, 1.0)
    dy = abs(y + 0.5 - cy) / max(ry, 1.0)
    jag = 0.14 * (p.hash01(x, y, salt) - 0.5)
    return dx ** 3 + dy ** 3 <= 1.0 + jag


def cobble() -> bytes:
    """Seamless 32px cobble: stones wrap so neighboring tiles continue the same rock."""
    buf = p.canvas(32, 32)
    mottled(buf, 32, 32, 21, p.DIRT, p.DIRT_DK, p.DIRT_HI)
    stones = [
        (3, 5, 7, 5),
        (17, 3, 6, 4),
        (29, 8, 7, 5),
        (7, 16, 6, 5),
        (21, 15, 7, 4),
        (4, 26, 6, 5),
        (18, 27, 7, 5),
        (30, 22, 6, 4),
        (12, 9, 4, 3),
        (25, 30, 5, 4),
        (0, 20, 5, 4),
    ]
    for i, (cx, cy, rx, ry) in enumerate(stones):
        for y in range(cy - ry - 3, cy + ry + 4):
            for x in range(cx - rx - 3, cx + rx + 4):
                if in_cobble(x, y, cx, cy, rx, ry, 40 + i * 3):
                    nx = (x - cx) / max(rx, 1)
                    ny = (y - cy) / max(ry, 1)
                    light = -nx * 0.55 - ny * 0.50
                    if light > 0.22:
                        col = p.STONE_HI
                    elif light < -0.22:
                        col = p.STONE_DK
                    else:
                        col = p.STONE
                    p.put_wrap(buf, 32, x, y, col)
    return bytes(buf)


def water_tile(salt: int, deep: bool) -> bytes:
    """Palette source only. Pond pixels are world-hashed in bake, not this stamp."""
    buf = p.canvas(32, 32)
    mid = p.WATER_DK if deep else p.WATER_MID
    dk = p.WATER_SH if deep else p.WATER_DK
    hi = p.WATER_MID if deep else p.WATER_HI
    mottled(buf, 32, 32, salt, mid, dk, hi, hi_chance=0.02)
    return bytes(buf)


def sand_tile() -> bytes:
    buf = p.canvas(32, 32)
    mottled(buf, 32, 32, 33, p.SAND, p.SAND_DK, p.SAND_HI)
    return bytes(buf)


def pebble() -> bytes:
    buf = p.canvas(16, 16)
    p.ellipse(buf, 16, 16, 8, 9, 4, 3, p.STONE_DK)
    p.ellipse(buf, 16, 16, 7, 8, 3, 2, p.STONE)
    p.ellipse(buf, 16, 16, 6, 7, 1, 1, p.STONE_HI)
    return bytes(buf)


def tuft() -> bytes:
    buf = p.canvas(16, 16)
    p.slynyrd_pad(buf, 16, 16, 8, 8, 6, 5, 3)
    return bytes(buf)


def main() -> None:
    ROOT.mkdir(parents=True, exist_ok=True)
    p.write_png(ROOT / "grass_a.png", 32, 32, grass(11))
    p.write_png(ROOT / "grass_b.png", 32, 32, grass(19))
    p.write_png(ROOT / "dirt.png", 32, 32, dirt_tile())
    p.write_png(ROOT / "stone.png", 32, 32, cobble())
    p.write_png(ROOT / "sand.png", 32, 32, sand_tile())
    p.write_png(ROOT / "water_deep.png", 32, 32, water_tile(41, True))
    p.write_png(ROOT / "water_shallow.png", 32, 32, water_tile(57, False))
    p.write_png(ROOT / "pebble.png", 16, 16, pebble())
    p.write_png(ROOT / "tuft.png", 16, 16, tuft())
    print(f"wrote tiles to {ROOT}")


if __name__ == "__main__":
    main()
