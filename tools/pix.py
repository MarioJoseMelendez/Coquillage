"""Shared 32px-tile pixel-art kit. Minecraft HD leaf/bark language, 32px tiles."""

from __future__ import annotations

import math
import pathlib
import struct
import zlib

# One forest ramp. Grass tiles and leaf clusters share LEAF_MID.
LEAF_HI = (186, 196, 72, 255)
LEAF_MID = (78, 126, 52, 255)
LEAF_MID2 = (58, 108, 46, 255)
LEAF_DK = (36, 78, 38, 255)
LEAF_SH = (56, 40, 72, 255)
SPRUCE_HI = (140, 168, 70, 255)
SPRUCE_MID = (36, 92, 48, 255)
SPRUCE_DK = (16, 56, 36, 255)
BARK_HI = (156, 112, 58, 255)
BARK = (120, 84, 44, 255)
BARK_DK = (76, 48, 44, 255)
PINE_BARK = (118, 62, 48, 255)
PINE_BARK_DK = (72, 36, 40, 255)
BIRCH = (214, 214, 204, 255)
BIRCH_DK = (40, 36, 44, 255)
DIRT = (148, 104, 62, 255)
DIRT_DK = (118, 80, 48, 255)
DIRT_HI = (176, 128, 78, 255)
SAND = (210, 192, 132, 255)
SAND_DK = (192, 170, 112, 255)
SAND_HI = (228, 212, 150, 255)
STONE = (168, 158, 148, 255)
STONE_HI = (214, 206, 190, 255)
STONE_DK = (70, 66, 64, 255)
GROUT = (92, 88, 84, 255)
PETAL_A = (214, 92, 120, 255)
PETAL_B = (236, 236, 210, 255)
GOLD = (236, 214, 70, 255)
SHROOM = (176, 48, 54, 255)
WATER_HI = (168, 206, 186, 255)
WATER_MID = (46, 112, 120, 255)
WATER_DK = (28, 62, 82, 255)
WATER_SH = (44, 36, 78, 255)
FOAM = (214, 224, 210, 255)


def crc(tag: bytes, data: bytes) -> int:
    return zlib.crc32(tag + data) & 0xFFFFFFFF


def chunk(tag: bytes, data: bytes) -> bytes:
    return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", crc(tag, data))


def write_png(path: pathlib.Path, width: int, height: int, rgba: bytes) -> None:
    expected = width * height * 4
    if len(rgba) != expected:
        raise ValueError(f"{path.name}: expected {expected} bytes, got {len(rgba)}")
    raw = b"".join(b"\x00" + rgba[y * width * 4 : (y + 1) * width * 4] for y in range(height))
    ihdr = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    png = (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", ihdr)
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )
    tmp = path.with_suffix(path.suffix + ".tmp")
    tmp.write_bytes(png)
    tmp.replace(path)


def canvas(width: int, height: int) -> bytearray:
    return bytearray(width * height * 4)


def put(buf: bytearray, width: int, x: int, y: int, color) -> None:
    height = len(buf) // (width * 4)
    px = bytes(color)
    if len(px) == 3:
        px += b"\xff"
    if 0 <= x < width and 0 <= y < height:
        i = (y * width + x) * 4
        buf[i : i + 4] = px


def put_wrap(buf: bytearray, width: int, x: int, y: int, color) -> None:
    height = len(buf) // (width * 4)
    put(buf, width, x % width, y % height, color)


def fill(buf: bytearray, width: int, height: int, color) -> None:
    for y in range(height):
        for x in range(width):
            put(buf, width, x, y, color)


def disk(buf, w, h, cx, cy, r, color) -> None:
    ellipse(buf, w, h, cx, cy, r, r, color)


def ellipse(buf, w, h, cx, cy, rx, ry, color) -> None:
    cx, cy = int(round(cx)), int(round(cy))
    rx, ry = max(int(round(rx)), 1), max(int(round(ry)), 1)
    for y in range(cy - ry - 1, cy + ry + 2):
        for x in range(cx - rx - 1, cx + rx + 2):
            nx = (x + 0.5 - cx) / rx
            ny = (y + 0.5 - cy) / ry
            if nx * nx + ny * ny <= 1.0:
                put(buf, w, x, y, color)


def star(buf, w, h, x: int, y: int, color) -> None:
    """Sunlit leaf fleck — wilderness bushes / Slynyrd 160 broccoli top."""
    for dx, dy in ((0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)):
        put(buf, w, x + dx, y + dy, color)


def leaf_nub(buf, w, h, x: int, y: int, color, salt: int) -> None:
    """Minecraft HD leaf: tiny rectangle, not a painted ball."""
    ww = 2 + (salt % 2)
    hh = 2
    for dy in range(hh):
        for dx in range(ww):
            put(buf, w, int(x) + dx, int(y) + dy, color)


def slynyrd_pad(buf, w, h, cx, cy, rx, ry, salt: int, hi=LEAF_HI, mid=LEAF_MID, dk=LEAF_DK, sh=LEAF_SH) -> None:
    """Slynyrd 160: volume rings + broccoli teeth. Minecraft nubs on top. Purple = SE crescent only."""
    cx, cy = float(cx), float(cy)
    rx, ry = max(float(rx), 3.0), max(float(ry), 3.0)
    x0, x1 = int(cx - rx - 6), int(cx + rx + 6)
    y0, y1 = int(cy - ry - 6), int(cy + ry + 6)
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            nx = (x + 0.5 - cx) / rx
            ny = (y + 0.5 - cy) / ry
            d = nx * nx + ny * ny
            ang = math.atan2(ny, nx)
            jag = 0.10 * math.sin(ang * 6.0 + salt * 0.4) + 0.05 * (hash01(x, y, salt) - 0.5)
            if d > 1.0 + jag:
                continue
            light = -nx * 0.55 - ny * 0.50
            if light < -0.52 or (d > 0.78 and light < -0.28):
                col = sh
            elif light < -0.08:
                col = dk
            elif light < 0.42:
                col = mid
            else:
                col = hi
            put(buf, w, x, y, col)

    nubs = max(14, int(rx * 1.6))
    for i in range(nubs):
        jitter = (hash01(i, salt, 1) - 0.5) * 0.35
        ang = (i / nubs) * math.tau + salt * 0.11 + jitter
        dist = 0.86 + hash01(i, salt, 2) * 0.22
        px = cx + math.cos(ang) * rx * dist
        py = cy + math.sin(ang) * ry * dist
        se = math.cos(ang) + math.sin(ang) > 0.35
        nw = math.cos(ang) + math.sin(ang) < -0.35
        col = sh if se else hi if nw else (mid if i % 2 == 0 else dk)
        rr = 3 + int(hash01(i, salt, 3) * 2)
        ellipse(buf, w, h, px, py, rr, rr - 1, col)

    for i in range(int(rx * ry * 0.38)):
        u = hash01(i, salt, 5)
        v = hash01(i, salt, 6)
        ang = u * math.tau
        rad = math.sqrt(max(v, 0.0)) * 0.78
        px = cx + math.cos(ang) * rx * rad
        py = cy + math.sin(ang) * ry * rad
        nx = (px - cx) / rx
        ny = (py - cy) / ry
        light = -nx * 0.55 - ny * 0.50
        if light > 0.40:
            col = hi
        elif light < -0.28:
            col = dk
        else:
            col = mid if hash01(i, salt, 9) > 0.35 else dk
        leaf_nub(buf, w, h, int(px), int(py), col, salt + i)

    for i in range(max(10, int(rx * 1.1))):
        u = hash01(i, salt, 7)
        v = hash01(i, salt, 8)
        if u < 0.18:
            continue
        px = int(cx - rx * 0.48 + u * rx * 0.85)
        py = int(cy - ry * 0.52 + v * ry * 0.70)
        nx = (px + 0.5 - cx) / rx
        ny = (py + 0.5 - cy) / ry
        if nx * nx + ny * ny > 0.82:
            continue
        star(buf, w, h, px, py, hi)


def hash01(x: int, y: int, salt: int) -> float:
    n = (int(x) * 374761393) ^ (int(y) * 668265263) ^ int(salt)
    n = (n ^ (n >> 13)) * 1274126177
    return ((n ^ (n >> 16)) & 0xFFFF) / 65535.0


def in_blob(x: int, y: int, cx: float, cy: float, rx: float, ry: float, salt: int) -> bool:
    nx = (x + 0.5 - cx) / max(rx, 1.0)
    ny = (y + 0.5 - cy) / max(ry, 1.0)
    d = nx * nx + ny * ny
    ang = math.atan2(ny, nx)
    jag = 0.10 * math.sin(ang * 5.0 + salt * 0.4) + 0.07 * (hash01(x, y, salt) - 0.5)
    return d <= 1.0 + jag


def leaf_tex(x: int, y: int, nx: float, ny: float, salt: int, hi, mid, mid2, dk, sh, density: float = 0.10):
    """Minecraft HD leaf block: 2×2 cells, holes, NW sun, purple SE. Not salt, not balloons."""
    cx2, cy2 = x // 2, y // 2
    hole = hash01(cx2, cy2, salt + 9)
    edge = min(1.0, math.hypot(nx, ny))
    if hole < density + edge * 0.14:
        return None
    n = hash01(cx2, cy2, salt + 1)
    light = -nx * 0.5 - ny * 0.45
    if light > 0.22:
        return sh if n > 0.5 else dk
    if light < -0.18 and n > 0.42:
        return hi
    if n < 0.28:
        return dk
    if n > 0.72:
        return mid
    return mid2


def fill_canopy(
    buf, w, h, blobs, salt: int, hi=LEAF_HI, mid=LEAF_MID, mid2=LEAF_MID2, dk=LEAF_DK, sh=LEAF_SH, density: float = 0.10
) -> None:
    for y in range(h):
        for x in range(w):
            hit = None
            for cx, cy, rx, ry in blobs:
                if in_blob(x, y, cx, cy, rx, ry, salt):
                    hit = (cx, cy, rx, ry)
                    break
            if hit is None:
                continue
            cx, cy, rx, ry = hit
            nx = (x - cx) / max(rx, 1.0)
            ny = (y - cy) / max(ry, 1.0)
            col = leaf_tex(x, y, nx, ny, salt, hi, mid, mid2, dk, sh, density)
            if col is not None:
                put(buf, w, x, y, col)


def leaf_cluster(buf, w, h, cx, cy, r, hi=LEAF_HI, mid=LEAF_MID, dk=LEAF_DK, sh=LEAF_SH) -> None:
    slynyrd_pad(buf, w, h, cx, cy, r, r * 0.86, 11, hi, mid, dk, sh)


def flare(buf, w, h, cx, cy, ang_deg: float, length: int, thick: int, c0, c1) -> None:
    """Slynyrd 160 root wedge: solid taper, not a 1px scratch."""
    ang = math.radians(ang_deg)
    dx, dy = math.cos(ang), math.sin(ang)
    length = max(int(length), 2)
    for i in range(length):
        t = i / (length - 1)
        r = max(1, int(round(thick * (1.0 - t * 0.72))))
        ellipse(
            buf,
            w,
            h,
            cx + dx * i,
            cy + dy * i,
            r,
            max(1, r - (1 if t > 0.45 else 0)),
            c0 if t < 0.4 else c1,
        )


def trunk(buf, w, h, cx, foot, shaft, half=3, light=BARK, dark=BARK_DK, birch: bool = False, roots: bool = True) -> None:
    """Slynyrd 160 tapered stump + wedges. Minecraft grain / birch dashes. Hue-shift like the canopy."""
    cx, foot, shaft, half = int(cx), int(foot), int(shaft), max(int(half), 2)
    hi = BARK_HI if not birch else (232, 232, 220, 255)
    ellipse(buf, w, h, cx + 2, foot + 3, half + 6, 4, LEAF_SH)
    top = foot - shaft
    for y in range(top, foot + 1):
        t = (y - top) / max(shaft, 1)
        hw = max(2, int(round(half * (0.58 + 0.42 * t))))
        for dx in range(-hw, hw + 1):
            u = dx / max(hw, 1)
            n = hash01(cx + dx, y, 3)
            if birch:
                col = hi if u < -0.4 else light if u < 0.35 else (176, 176, 168, 255)
                if (y + dx * 2) % 6 == 0:
                    col = dark
            else:
                if u < -0.42:
                    col = hi
                elif u > 0.38:
                    col = dark
                else:
                    col = light
                if n < 0.10 or (dx + y) % 5 == 0:
                    col = dark
            put(buf, w, cx + dx, y, col)
        if not birch and y % 5 == 1:
            put(buf, w, cx + hw, y, (42, 28, 32, 255))
    if not birch:
        ellipse(buf, w, h, cx - 1, foot - shaft * 0.45, 2, 2, dark)
        ellipse(buf, w, h, cx + 2, foot - shaft * 0.72, 1, 2, dark)
    if roots:
        for ang, length, thick in (
            (90, 9, 4),
            (38, 8, 3),
            (142, 8, 3),
            (8, 7, 3),
            (172, 7, 3),
        ):
            flare(buf, w, h, cx, foot, ang, length, thick, light if not birch else BARK, dark if not birch else BARK_DK)
