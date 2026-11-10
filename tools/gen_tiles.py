"""Minecraft-style 32×32 terrain: clustered shapes, tiny palettes, no salt-and-pepper."""

from __future__ import annotations

import math
import pathlib
import struct
import zlib

ROOT = pathlib.Path(__file__).resolve().parents[1] / "assets" / "terrain"


def crc(tag: bytes, data: bytes) -> int:
    return zlib.crc32(tag + data) & 0xFFFFFFFF


def chunk(tag: bytes, data: bytes) -> bytes:
    return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", crc(tag, data))


def write_png(path: pathlib.Path, width: int, height: int, rgba: bytes) -> None:
    raw = b"".join(b"\x00" + rgba[y * width * 4 : (y + 1) * width * 4] for y in range(height))
    ihdr = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    png = (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", ihdr)
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )
    path.write_bytes(png)


def hash32(x: int, y: int, salt: int) -> int:
    n = (x * 374761393) ^ (y * 668265263) ^ salt
    n = (n ^ (n >> 13)) * 1274126177
    return (n ^ (n >> 16)) & 0xFFFFFFFF


def put(buf: bytearray, width: int, x: int, y: int, color: tuple[int, int, int, int]) -> None:
    height = len(buf) // (width * 4)
    if 0 <= x < width and 0 <= y < height:
        i = (y * width + x) * 4
        buf[i : i + 4] = bytes(color)


def fill(buf: bytearray, width: int, height: int, color: tuple[int, int, int, int]) -> None:
    for y in range(height):
        for x in range(width):
            put(buf, width, x, y, color)


def stamp_blob(
    buf: bytearray,
    width: int,
    height: int,
    cx: float,
    cy: float,
    rx: float,
    ry: float,
    color: tuple[int, int, int, int],
) -> None:
    for y in range(height):
        for x in range(width):
            nx = (x + 0.5 - cx) / rx
            ny = (y + 0.5 - cy) / ry
            if nx * nx + ny * ny <= 1.0:
                put(buf, width, x, y, color)


def grass(base, dark, mid, blade, salt: int) -> bytes:
    buf = bytearray(32 * 32 * 4)
    fill(buf, 32, 32, (*base, 255))
    patches = [
        (8 + (hash32(0, 0, salt) % 5), 7, 7, 5, dark),
        (22, 10 + (hash32(1, 0, salt) % 4), 6, 5, dark),
        (14, 22, 8, 6, mid),
        (26, 24, 5, 4, dark),
    ]
    for cx, cy, rx, ry, color in patches:
        stamp_blob(buf, 32, 32, cx, cy, rx, ry, (*color, 255))
    blades = [
        (4, 18, 5),
        (6, 20, 4),
        (18, 6, 4),
        (20, 8, 3),
        (27, 16, 4),
        (11, 27, 3),
    ]
    for x, y0, length in blades:
        x += (hash32(x, y0, salt) % 3) - 1
        for i in range(length):
            put(buf, 32, x, y0 - i, (*blade, 255))
            if i > 1:
                put(buf, 32, x + 1, y0 - i, (*mid, 255))
    return bytes(buf)


def dirt_tile() -> bytes:
    buf = bytearray(32 * 32 * 4)
    fill(buf, 32, 32, (150, 108, 70, 255))
    stamp_blob(buf, 32, 32, 9, 11, 8, 6, (120, 82, 50, 255))
    stamp_blob(buf, 32, 32, 22, 20, 7, 6, (128, 88, 54, 255))
    stamp_blob(buf, 32, 32, 18, 7, 5, 4, (168, 124, 82, 255))
    # pebble
    stamp_blob(buf, 32, 32, 11, 24, 2.4, 1.8, (98, 86, 72, 255))
    put(buf, 32, 10, 23, (168, 156, 132, 255))
    # short root
    for x in range(20, 27):
        put(buf, 32, x, 27 + (x % 3 == 0), (92, 62, 40, 255))
    return bytes(buf)


def cobble() -> bytes:
    """Irregular stones like Minecraft cobblestone, not a grid."""
    buf = bytearray(32 * 32 * 4)
    grout = (58, 54, 50, 255)
    fill(buf, 32, 32, grout)
    seeds = []
    for i in range(12):
        sx = 3 + (hash32(i, 0, 71) % 26)
        sy = 3 + (hash32(i, 1, 73) % 26)
        seeds.append((sx, sy, i))
    stones = [
        (118, 112, 104),
        (132, 126, 116),
        (108, 104, 98),
        (140, 132, 120),
        (124, 118, 108),
    ]
    for y in range(32):
        for x in range(32):
            best = second = 1e9
            nearest = 0
            nsx, nsy = seeds[0][0], seeds[0][1]
            for sx, sy, idx in seeds:
                d = math.hypot(x - sx, y - sy)
                if d < best:
                    second = best
                    best = d
                    nearest = idx
                    nsx, nsy = sx, sy
                elif d < second:
                    second = d
            if second - best < 1.35:
                put(buf, 32, x, y, grout)
                continue
            base = stones[nearest % len(stones)]
            if x - nsx <= -1 and y - nsy <= 0:
                base = (min(255, base[0] + 28), min(255, base[1] + 26), min(255, base[2] + 22))
            elif x - nsx >= 2 and y - nsy >= 1:
                base = (max(0, base[0] - 22), max(0, base[1] - 20), max(0, base[2] - 18))
            put(buf, 32, x, y, (*base, 255))
    return bytes(buf)


def sand_tile() -> bytes:
    buf = bytearray(32 * 32 * 4)
    fill(buf, 32, 32, (210, 196, 138, 255))
    stamp_blob(buf, 32, 32, 10, 12, 9, 6, (198, 180, 118, 255))
    stamp_blob(buf, 32, 32, 23, 22, 8, 5, (222, 208, 150, 255))
    stamp_blob(buf, 32, 32, 18, 8, 4, 3, (186, 166, 104, 255))
    return bytes(buf)


def pebble() -> bytes:
    buf = bytearray(16 * 16 * 4)
    stamp_blob(buf, 16, 16, 8, 9, 5.0, 3.6, (108, 100, 90, 255))
    stamp_blob(buf, 16, 16, 7, 8, 2.2, 1.6, (168, 160, 146, 255))
    put(buf, 16, 10, 10, (82, 76, 70, 255))
    return bytes(buf)


def tuft() -> bytes:
    buf = bytearray(16 * 16 * 4)
    blades = [(6, 14, 7), (8, 14, 8), (10, 14, 6), (7, 14, 5)]
    for x, y0, length in blades:
        for i in range(length):
            color = (86, 140, 52, 255) if i > 2 else (48, 86, 36, 255)
            put(buf, 16, x, y0 - i, color)
            if i > 3:
                put(buf, 16, x + 1, y0 - i, (70, 118, 44, 255))
    return bytes(buf)


def main() -> None:
    ROOT.mkdir(parents=True, exist_ok=True)
    write_png(
        ROOT / "grass_a.png",
        32,
        32,
        grass((74, 124, 48), (52, 92, 36), (64, 108, 42), (92, 148, 56), 11),
    )
    write_png(
        ROOT / "grass_b.png",
        32,
        32,
        grass((58, 108, 52), (40, 82, 40), (50, 96, 46), (78, 132, 60), 29),
    )
    write_png(ROOT / "dirt.png", 32, 32, dirt_tile())
    write_png(ROOT / "stone.png", 32, 32, cobble())
    write_png(ROOT / "sand.png", 32, 32, sand_tile())
    write_png(ROOT / "pebble.png", 16, 16, pebble())
    write_png(ROOT / "tuft.png", 16, 16, tuft())
    print(f"wrote tiles to {ROOT}")


if __name__ == "__main__":
    main()
