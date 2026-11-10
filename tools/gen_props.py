"""Top-down vegetation in Minecraft HD language: log-top + leaf puffs, not noise."""

from __future__ import annotations

import math
import pathlib
import struct
import zlib

ROOT = pathlib.Path(__file__).resolve().parents[1] / "assets" / "props"


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


def put(buf: bytearray, width: int, x: int, y: int, color: tuple[int, int, int, int]) -> None:
    height = len(buf) // (width * 4)
    if 0 <= x < width and 0 <= y < height:
        i = (y * width + x) * 4
        buf[i : i + 4] = bytes(color)


def fill_ellipse(
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
            nx = (x + 0.5 - cx) / max(rx, 0.5)
            ny = (y + 0.5 - cy) / max(ry, 0.5)
            if nx * nx + ny * ny <= 1.0:
                put(buf, width, x, y, color)


def outline_ellipse(
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
            nx = (x + 0.5 - cx) / max(rx, 0.5)
            ny = (y + 0.5 - cy) / max(ry, 0.5)
            d = nx * nx + ny * ny
            if 0.82 <= d <= 1.05:
                put(buf, width, x, y, color)


def leaf_diamond(
    buf: bytearray,
    width: int,
    height: int,
    cx: int,
    cy: int,
    size: int,
    color: tuple[int, int, int, int],
) -> None:
    for y in range(-size, size + 1):
        for x in range(-size, size + 1):
            if abs(x) + abs(y) <= size:
                put(buf, width, cx + x, cy + y, color)


def log_top(buf: bytearray, width: int, height: int, cx: int, cy: int, radius: int) -> None:
    """Minecraft oak log end: concentric rings."""
    bark = (86, 62, 34, 255)
    ring = (138, 102, 54, 255)
    inner = (168, 128, 70, 255)
    pith = (196, 160, 96, 255)
    fill_ellipse(buf, width, height, cx, cy, radius + 0.5, radius + 0.5, bark)
    fill_ellipse(buf, width, height, cx, cy, radius - 1.2, radius - 1.2, ring)
    fill_ellipse(buf, width, height, cx, cy, max(2.0, radius - 3.2), max(2.0, radius - 3.2), inner)
    fill_ellipse(buf, width, height, cx, cy, 1.6, 1.6, pith)
    # four bark cracks like vanilla log-top
    for dx, dy in ((0, -radius + 1), (0, radius - 1), (-radius + 1, 0), (radius - 1, 0)):
        put(buf, width, cx + dx, cy + dy, (62, 42, 22, 255))


def thick_line(
    buf: bytearray,
    width: int,
    height: int,
    x0: float,
    y0: float,
    x1: float,
    y1: float,
    thickness: int,
    color: tuple[int, int, int, int],
) -> None:
    steps = int(max(abs(x1 - x0), abs(y1 - y0))) + 1
    for i in range(steps):
        t = i / max(steps - 1, 1)
        x = int(round(x0 + (x1 - x0) * t))
        y = int(round(y0 + (y1 - y0) * t))
        r2 = thickness * thickness
        for oy in range(-thickness, thickness + 1):
            for ox in range(-thickness, thickness + 1):
                if ox * ox + oy * oy <= r2:
                    put(buf, width, x + ox, y + oy, color)


def tree(width: int, height: int, arms: list[tuple], dark, mid, light) -> bytes:
    """Top-down tree: trunk rings + radiating branches + leaf clumps at the tips."""
    buf = bytearray(width * height * 4)
    cx, cy = width / 2, height / 2
    wood = (92, 64, 34, 255)
    wood_dark = (62, 42, 22, 255)
    # branches first so the silhouette reads as a tree, not a green blob
    tips = []
    for angle, length, puff_r in arms:
        x1 = cx + math.cos(angle) * length
        y1 = cy + math.sin(angle) * length
        thick_line(buf, width, height, cx, cy, x1, y1, 2, wood_dark)
        thick_line(buf, width, height, cx, cy, x1, y1, 1, wood)
        fork = 0.58
        fx = cx + math.cos(angle) * length * fork
        fy = cy + math.sin(angle) * length * fork
        fa = angle + 0.55
        x2 = fx + math.cos(fa) * length * 0.32
        y2 = fy + math.sin(fa) * length * 0.32
        thick_line(buf, width, height, fx, fy, x2, y2, 1, wood)
        tips.append((x1, y1, puff_r))
        tips.append((x2, y2, puff_r * 0.65))
    for px, py, pr in tips:
        fill_ellipse(buf, width, height, px, py, pr + 1.2, pr * 0.9 + 1.0, (*dark, 255))
        fill_ellipse(buf, width, height, px, py, pr * 0.72, pr * 0.62, (*mid, 255))
        fill_ellipse(
            buf,
            width,
            height,
            px - pr * 0.2,
            py - pr * 0.2,
            pr * 0.32,
            pr * 0.28,
            (*light, 255),
        )
        leaf_diamond(buf, width, height, int(px + pr * 0.4), int(py), 2, (*mid, 255))
    log_top(buf, width, height, int(cx), int(cy), 8 if width >= 80 else 6)
    return bytes(buf)


def bush(width: int, height: int, puffs, dark, mid, light) -> bytes:
    buf = bytearray(width * height * 4)
    for px, py, rx, ry in puffs:
        fill_ellipse(buf, width, height, px, py, rx + 1, ry + 1, (*dark, 255))
    for px, py, rx, ry in puffs:
        fill_ellipse(buf, width, height, px, py, rx * 0.75, ry * 0.75, (*mid, 255))
    px, py, rx, ry = puffs[0]
    fill_ellipse(buf, width, height, px - 1, py - 1, rx * 0.4, ry * 0.35, (*light, 255))
    return bytes(buf)


def fern(width: int, height: int, arms: int) -> bytes:
    buf = bytearray(width * height * 4)
    cx, cy = width * 0.5, height * 0.5
    radius = min(width, height) * 0.46
    dark, mid, tip = (36, 78, 36, 255), (52, 108, 44, 255), (78, 140, 56, 255)
    for arm in range(arms):
        a0 = (arm / arms) * math.tau
        length = radius * (0.85 if arm % 2 == 0 else 0.72)
        for r in range(int(length)):
            a = a0 + math.sin(r * 0.35) * 0.08
            x = int(cx + math.cos(a) * r)
            y = int(cy + math.sin(a) * r)
            color = tip if r > length * 0.65 else mid if r > 3 else dark
            put(buf, width, x, y, color)
            if r < length * 0.55 and r % 3 == 0:
                put(buf, width, int(cx + math.cos(a + 0.4) * (r * 0.7)), int(cy + math.sin(a + 0.4) * (r * 0.7)), mid)
                put(buf, width, int(cx + math.cos(a - 0.4) * (r * 0.7)), int(cy + math.sin(a - 0.4) * (r * 0.7)), mid)
    put(buf, width, int(cx), int(cy), dark)
    return bytes(buf)


def flower(width: int, height: int, petal: tuple[int, int, int]) -> bytes:
    buf = bytearray(width * height * 4)
    cx, cy = width // 2, height // 2 + 1
    put(buf, width, cx, cy + 3, (48, 92, 36, 255))
    put(buf, width, cx, cy + 4, (48, 92, 36, 255))
    put(buf, width, cx - 1, cy + 2, (64, 112, 44, 255))
    for dx, dy in ((0, -2), (0, 2), (2, 0), (-2, 0), (1, -1), (-1, -1), (1, 1), (-1, 1)):
        put(buf, width, cx + dx, cy + dy, (*petal, 255))
    put(buf, width, cx, cy, (236, 214, 70, 255))
    put(buf, width, cx, cy - 1, (*petal, 255))
    return bytes(buf)


def mushroom() -> bytes:
    buf = bytearray(16 * 16 * 4)
    fill_ellipse(buf, 16, 16, 8, 8, 5.2, 4.2, (176, 48, 44, 255))
    fill_ellipse(buf, 16, 16, 8, 8, 3.2, 2.4, (198, 70, 60, 255))
    fill_ellipse(buf, 16, 16, 8, 8, 1.4, 1.2, (232, 214, 186, 255))
    put(buf, 16, 5, 7, (236, 236, 220, 255))
    put(buf, 16, 6, 6, (236, 236, 220, 255))
    put(buf, 16, 11, 9, (236, 236, 220, 255))
    return bytes(buf)


def clump() -> bytes:
    buf = bytearray(24 * 16 * 4)
    for x, h, color in (
        (6, 8, (48, 92, 36, 255)),
        (8, 11, (70, 124, 48, 255)),
        (10, 9, (48, 92, 36, 255)),
        (12, 12, (86, 140, 52, 255)),
        (14, 8, (70, 124, 48, 255)),
        (16, 7, (48, 92, 36, 255)),
    ):
        for i in range(h):
            put(buf, 24, x, 15 - i, color)
            if i > h // 2:
                put(buf, 24, x + 1, 15 - i, (70, 124, 48, 255))
    return bytes(buf)


def main() -> None:
    ROOT.mkdir(parents=True, exist_ok=True)
    oak_d, oak_m, oak_l = (28, 70, 22), (46, 108, 34), (78, 148, 48)
    spruce_d, spruce_m, spruce_l = (16, 56, 36), (28, 86, 52), (48, 120, 70)
    birch_d, birch_m, birch_l = (48, 86, 22), (78, 124, 36), (124, 168, 52)

    write_png(
        ROOT / "tree_a.png",
        96,
        96,
        tree(
            96,
            96,
            [
                (0.15, 38, 13),
                (0.95, 36, 12),
                (1.85, 40, 14),
                (2.55, 34, 12),
                (3.45, 38, 13),
                (4.40, 32, 11),
                (5.20, 36, 12),
            ],
            oak_d,
            oak_m,
            oak_l,
        ),
    )
    write_png(
        ROOT / "tree_b.png",
        80,
        96,
        tree(
            80,
            96,
            [
                (1.57, 42, 11),
                (0.70, 34, 10),
                (2.40, 34, 10),
                (0.10, 28, 9),
                (3.05, 30, 9),
                (4.20, 26, 8),
                (5.40, 26, 8),
            ],
            spruce_d,
            spruce_m,
            spruce_l,
        ),
    )
    write_png(
        ROOT / "tree_c.png",
        72,
        72,
        tree(
            72,
            72,
            [
                (0.4, 26, 10),
                (1.6, 28, 11),
                (2.8, 24, 9),
                (4.1, 27, 10),
                (5.3, 24, 9),
            ],
            birch_d,
            birch_m,
            birch_l,
        ),
    )
    write_png(
        ROOT / "bush_a.png",
        48,
        48,
        bush(
            48,
            48,
            [(24, 26, 18, 14), (14, 24, 10, 9), (34, 24, 10, 9), (24, 16, 9, 8)],
            oak_d,
            oak_m,
            oak_l,
        ),
    )
    write_png(
        ROOT / "bush_b.png",
        40,
        32,
        bush(
            40,
            32,
            [(20, 18, 16, 11), (10, 18, 8, 7), (30, 17, 8, 7)],
            birch_d,
            birch_m,
            birch_l,
        ),
    )
    write_png(
        ROOT / "bush_c.png",
        32,
        32,
        bush(32, 32, [(16, 18, 12, 10), (10, 16, 7, 6), (22, 16, 7, 6)], oak_d, oak_m, oak_l),
    )
    write_png(ROOT / "fern_a.png", 32, 32, fern(32, 32, 6))
    write_png(ROOT / "fern_b.png", 40, 32, fern(40, 32, 7))
    write_png(ROOT / "flower_a.png", 16, 16, flower(16, 16, (214, 92, 120)))
    write_png(ROOT / "flower_b.png", 16, 16, flower(16, 16, (236, 236, 210)))
    write_png(ROOT / "mushroom_a.png", 16, 16, mushroom())
    write_png(ROOT / "clump_a.png", 24, 16, clump())
    print(f"wrote props to {ROOT}")


if __name__ == "__main__":
    main()
