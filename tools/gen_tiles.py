"""Generate Phase 2 terrain PNGs. Pixel density is 1 world pixel = 1 sprite pixel."""

from __future__ import annotations

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
    i = (y * width + x) * 4
    buf[i : i + 4] = bytes(color)


def grass_a(x: int, y: int) -> tuple[int, int, int, int]:
    n = hash32(x, y, 11) % 100
    if hash32(x, y, 3) % 17 == 0:
        return (58, 96, 46, 255)
    if n < 18:
        return (32, 54, 34, 255)
    if n < 50:
        return (40, 68, 38, 255)
    return (48, 78, 42, 255)


def grass_b(x: int, y: int) -> tuple[int, int, int, int]:
    n = hash32(x, y, 29) % 100
    if hash32(x, y, 7) % 21 == 0:
        return (90, 108, 40, 255)
    if n < 28:
        return (28, 50, 30, 255)
    return (38, 62, 36, 255)


def dirt(x: int, y: int) -> tuple[int, int, int, int]:
    n = hash32(x, y, 41) % 100
    if hash32(x, y, 19) % 29 == 0:
        return (108, 88, 70, 255)
    if n < 22:
        return (74, 50, 34, 255)
    if n < 58:
        return (92, 64, 40, 255)
    return (110, 78, 48, 255)


def stone(x: int, y: int) -> tuple[int, int, int, int]:
    cell_x, cell_y = x // 8, y // 8
    local_x, local_y = x % 8, y % 8
    if local_x == 0 or local_y == 0:
        return (52, 46, 42, 255)
    if local_x == 2 and local_y == 2:
        return (168, 158, 140, 255)
    n = hash32(cell_x, cell_y, 53) % 4
    return (
        (126, 118, 108, 255),
        (114, 110, 102, 255),
        (138, 128, 114, 255),
        (120, 116, 106, 255),
    )[n]


def sand(x: int, y: int) -> tuple[int, int, int, int]:
    n = hash32(x, y, 67) % 100
    ripple = ((x + y // 2) % 7) == 0
    if hash32(x, y, 13) % 37 == 0:
        return (224, 204, 148, 255)
    if ripple:
        return (168, 140, 88, 255)
    if n < 25:
        return (186, 154, 96, 255)
    if n < 60:
        return (204, 174, 112, 255)
    return (214, 186, 124, 255)


def paint_tile(fn) -> bytes:
    buf = bytearray(32 * 32 * 4)
    for y in range(32):
        for x in range(32):
            put(buf, 32, x, y, fn(x, y))
    return bytes(buf)


def pebble() -> bytes:
    buf = bytearray(16 * 16 * 4)
    cx, cy, rx, ry = 7.5, 8.0, 5.2, 4.0
    for y in range(16):
        for x in range(16):
            nx = (x - cx) / rx
            ny = (y - cy) / ry
            if nx * nx + ny * ny <= 1.0:
                highlight = nx < -0.2 and ny < -0.15
                color = (168, 160, 148, 255) if highlight else (108, 100, 92, 255)
                if hash32(x, y, 9) % 5 == 0:
                    color = (88, 82, 76, 255)
                put(buf, 16, x, y, color)
    return bytes(buf)


def tuft() -> bytes:
    buf = bytearray(16 * 16 * 4)
    blades = [(7, 14, 4), (5, 13, 6), (9, 13, 5), (6, 12, 7), (8, 12, 6), (10, 14, 3)]
    for x, y0, length in blades:
        for i in range(length):
            y = y0 - i
            if 0 <= x < 16 and 0 <= y < 16:
                color = (70, 108, 46, 255) if i > 1 else (48, 78, 36, 255)
                put(buf, 16, x, y, color)
                if i > 2 and x + 1 < 16:
                    put(buf, 16, x + 1, y, (86, 118, 48, 255))
    return bytes(buf)


def main() -> None:
    ROOT.mkdir(parents=True, exist_ok=True)
    write_png(ROOT / "grass_a.png", 32, 32, paint_tile(grass_a))
    write_png(ROOT / "grass_b.png", 32, 32, paint_tile(grass_b))
    write_png(ROOT / "dirt.png", 32, 32, paint_tile(dirt))
    write_png(ROOT / "stone.png", 32, 32, paint_tile(stone))
    write_png(ROOT / "sand.png", 32, 32, paint_tile(sand))
    write_png(ROOT / "pebble.png", 16, 16, pebble())
    write_png(ROOT / "tuft.png", 16, 16, tuft())
    print(f"wrote tiles to {ROOT}")


if __name__ == "__main__":
    main()
