"""Vegetation from the approved refs: Slynyrd 124/160 pads, wilderness lime stars, Minecraft bark."""

from __future__ import annotations

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import pix as p

ROOT = pathlib.Path(__file__).resolve().parents[1] / "assets" / "props"


def oak() -> bytes:
    buf = p.canvas(96, 96)
    p.trunk(buf, 96, 96, 48, 90, 32, 7)
    p.slynyrd_pad(buf, 96, 96, 38, 48, 26, 22, 11)
    p.slynyrd_pad(buf, 96, 96, 58, 46, 24, 20, 17)
    return bytes(buf)


def conifer() -> bytes:
    buf = p.canvas(64, 96)
    p.trunk(buf, 64, 96, 32, 90, 74, 3, p.PINE_BARK, p.PINE_BARK_DK, roots=False)
    p.slynyrd_pad(buf, 64, 96, 32, 80, 24, 11, 31, p.SPRUCE_HI, p.SPRUCE_MID, p.SPRUCE_DK, p.LEAF_SH)
    p.slynyrd_pad(buf, 64, 96, 28, 66, 22, 10, 37, p.SPRUCE_HI, p.SPRUCE_MID, p.SPRUCE_DK, p.LEAF_SH)
    p.slynyrd_pad(buf, 64, 96, 36, 52, 20, 10, 41, p.SPRUCE_HI, p.SPRUCE_MID, p.SPRUCE_DK, p.LEAF_SH)
    p.slynyrd_pad(buf, 64, 96, 30, 38, 16, 9, 43, p.SPRUCE_HI, p.SPRUCE_MID, p.SPRUCE_DK, p.LEAF_SH)
    p.slynyrd_pad(buf, 64, 96, 34, 26, 12, 8, 47, p.SPRUCE_HI, p.SPRUCE_MID, p.SPRUCE_DK, p.LEAF_SH)
    p.slynyrd_pad(buf, 64, 96, 32, 16, 8, 6, 53, p.SPRUCE_HI, p.SPRUCE_MID, p.SPRUCE_DK, p.LEAF_SH)
    p.trunk(buf, 64, 96, 32, 90, 10, 3, p.PINE_BARK, p.PINE_BARK_DK, roots=False)
    return bytes(buf)


def birch() -> bytes:
    buf = p.canvas(64, 64)
    p.trunk(buf, 64, 64, 32, 58, 24, 4, p.BIRCH, p.BIRCH_DK, birch=True)
    p.slynyrd_pad(buf, 64, 64, 26, 30, 16, 14, 53, (210, 214, 90, 255), (90, 140, 48, 255), p.LEAF_DK, p.LEAF_SH)
    p.slynyrd_pad(buf, 64, 64, 40, 28, 14, 12, 59, (210, 214, 90, 255), (90, 140, 48, 255), p.LEAF_DK, p.LEAF_SH)
    return bytes(buf)


def bush(salt: int) -> bytes:
    buf = p.canvas(32, 32)
    # Stump only. Ground blob is the dynamic shadow sprite, not baked-in.
    p.ellipse(buf, 32, 32, 16, 26, 3, 3, p.BARK_DK)
    p.ellipse(buf, 32, 32, 16, 25, 2, 3, p.BARK)
    p.put(buf, 32, 15, 24, p.BARK_HI)
    p.slynyrd_pad(buf, 32, 32, 16, 14, 9, 8, salt)
    return bytes(buf)


def fern(salt: int) -> bytes:
    buf = p.canvas(32, 32)
    p.slynyrd_pad(buf, 32, 32, 13, 18, 6, 5, salt, p.SPRUCE_HI, p.SPRUCE_MID, p.SPRUCE_DK, p.LEAF_SH)
    p.slynyrd_pad(buf, 32, 32, 20, 17, 6, 5, salt + 3, p.SPRUCE_HI, p.SPRUCE_MID, p.SPRUCE_DK, p.LEAF_SH)
    p.slynyrd_pad(buf, 32, 32, 16, 12, 5, 4, salt + 5, p.SPRUCE_HI, p.SPRUCE_MID, p.SPRUCE_DK, p.LEAF_SH)
    return bytes(buf)


def flower(petal) -> bytes:
    buf = p.canvas(16, 16)
    p.put(buf, 16, 8, 12, p.LEAF_SH)
    p.put(buf, 16, 8, 11, p.LEAF_DK)
    p.put(buf, 16, 8, 10, p.LEAF_MID)
    for dx, dy in ((0, -2), (2, 0), (0, 2), (-2, 0), (1, -1), (-1, 1)):
        p.put(buf, 16, 8 + dx, 7 + dy, petal)
    p.put(buf, 16, 8, 7, p.GOLD)
    return bytes(buf)


def mushroom() -> bytes:
    buf = p.canvas(16, 16)
    p.ellipse(buf, 16, 16, 8, 11, 4, 2, p.LEAF_SH)
    p.ellipse(buf, 16, 16, 8, 8, 4, 3, p.SHROOM)
    p.put(buf, 16, 7, 7, (232, 214, 186, 255))
    p.put(buf, 16, 10, 9, (232, 214, 186, 255))
    p.put(buf, 16, 8, 11, p.DIRT_HI)
    p.put(buf, 16, 8, 12, p.DIRT_HI)
    return bytes(buf)


def clump() -> bytes:
    buf = p.canvas(32, 16)
    p.slynyrd_pad(buf, 32, 16, 16, 8, 11, 6, 7)
    return bytes(buf)


def hierba(width: int, height: int, salt: int, tall: bool) -> bytes:
    """Wild grass tuft (hierba), not lawn. Same 4-color ramp as the canopy."""
    buf = p.canvas(width, height)
    cx = width // 2
    base = height - 3
    p.ellipse(buf, width, height, cx, base - (4 if tall else 2), 7 if tall else 4, 5 if tall else 3, p.LEAF_DK)
    p.ellipse(buf, width, height, cx - 1, base - (7 if tall else 4), 5 if tall else 2, 4 if tall else 2, p.LEAF_MID)
    n = 5 if tall else 5
    for i in range(n):
        x = cx + int((p.hash01(i, salt, 1) - 0.5) * (width * 0.7))
        hgt = (14 + int(p.hash01(i, salt, 2) * 12)) if tall else (5 + int(p.hash01(i, salt, 2) * 6))
        lean = (p.hash01(i, salt, 3) - 0.5) * (4.0 if tall else 2.0)
        wide = p.hash01(i, salt, 5) > 0.45
        tip = base - hgt
        for y in range(tip, base - 1):
            t = (y - tip) / max(hgt, 1)
            ox = int(lean * t)
            if t < 0.16:
                col = p.LEAF_HI
            elif t < 0.52:
                col = p.LEAF_MID
            else:
                col = p.LEAF_DK
            p.put(buf, width, x + ox, y, col)
            if wide and t < 0.55:
                p.put(buf, width, x + ox + (1 if lean >= 0 else -1), y, col)
        if tall and p.hash01(i, salt, 4) > 0.62:
            p.star(buf, width, height, x + int(lean * 0.12), tip, p.LEAF_HI)
    return bytes(buf)


def _get(buf: bytearray, width: int, x: int, y: int):
    i = (y * width + x) * 4
    return tuple(buf[i : i + 4])


def flip_h(buf: bytes | bytearray, width: int, height: int) -> bytes:
    src = bytearray(buf)
    out = p.canvas(width, height)
    for y in range(height):
        for x in range(width):
            p.put(out, width, width - 1 - x, y, _get(src, width, x, y))
    return bytes(out)


def outline(buf: bytearray, width: int, height: int, col) -> bytearray:
    src = bytes(buf)
    out = p.canvas(width, height)
    for y in range(height):
        for x in range(width):
            if src[(y * width + x) * 4 + 3] < 20:
                continue
            for dx, dy in ((-1, 0), (1, 0), (0, -1), (0, 1)):
                p.put(out, width, x + dx, y + dy, col)
    for y in range(height):
        for x in range(width):
            i = (y * width + x) * 4
            if src[i + 3] >= 20:
                out[i : i + 4] = src[i : i + 4]
    return out


def player(facing: str, step: bool) -> bytes:
    """True 90° bird's-eye: head + shoulders + cloak. No face. Palette cloak = GOLD."""
    # Framesprite: 90° shows scalp, shoulders, footprint; face hidden.
    # Pixeli: N is hair/nape; S shows a hint of chest toward +Y. W = flip of E.
    # Hands: blocky SAND pads on the sides toward the front (Mario sample 2).
    w = h = 32
    buf = p.canvas(w, h)
    bob = 1 if step else 0
    if facing == "s":
        p.ellipse(buf, w, h, 16, 22 + bob, 9, 6, p.BARK)
        p.ellipse(buf, w, h, 16, 20 + bob, 9, 6, p.GOLD)
        p.ellipse(buf, w, h, 16, 12, 7, 7, p.LEAF_SH)
        p.ellipse(buf, w, h, 15, 11, 4, 4, p.BARK)
        p.put(buf, w, 16, 9, p.BARK_HI)
        p.put(buf, w, 16, 17, p.SAND)
        p.put(buf, w, 15, 17, p.SAND)
        p.put(buf, w, 16, 24 + bob, p.PETAL_A)
    elif facing == "n":
        p.ellipse(buf, w, h, 16, 22 - bob, 9, 6, p.BARK)
        p.ellipse(buf, w, h, 16, 20 - bob, 9, 6, p.GOLD)
        p.ellipse(buf, w, h, 16, 12, 7, 7, p.LEAF_SH)
        p.ellipse(buf, w, h, 16, 11, 5, 5, p.BARK_DK)
        p.put(buf, w, 15, 10, p.BARK)
        p.put(buf, w, 16, 8, p.BARK_HI)
        p.ellipse(buf, w, h, 16, 24, 5, 3, p.LEAF_SH)
    else:
        p.ellipse(buf, w, h, 16, 22, 7, 6, p.BARK)
        p.ellipse(buf, w, h, 16, 20, 7, 6, p.GOLD)
        p.ellipse(buf, w, h, 17, 12, 7, 7, p.LEAF_SH)
        p.ellipse(buf, w, h, 16, 11, 4, 4, p.BARK)
        p.put(buf, w, 17, 9, p.BARK_HI)
        p.put(buf, w, 17, 17, p.SAND)
        p.put(buf, w, 18, 24, p.PETAL_A)
    buf = outline(buf, w, h, p.LEAF_SH)
    stamp_hands(buf, w, facing, step)
    return bytes(buf)


def stamp_hands(buf: bytearray, w: int, facing: str, step: bool) -> None:
    bob = 1 if step else 0

    def paw(cx: int, cy: int) -> None:
        p.ellipse(buf, w, w, cx, cy, 5, 4, p.SAND)
        p.ellipse(buf, w, w, cx - 1, cy - 1, 3, 2, p.SAND_HI)
        p.put(buf, w, cx + 2, cy + 1, p.SAND_DK)
        p.put(buf, w, cx + 1, cy + 2, p.SAND_DK)

    if facing == "s":
        paw(6, 24 + bob)
        paw(25, 24 - bob)
    elif facing == "n":
        paw(6, 22 - bob)
        paw(25, 22 + bob)
    else:
        paw(25, 22 - bob)
        paw(7, 24 + bob)


def shadow(width: int, height: int) -> bytes:
    """Flat ground oval + denser contact at the caster foot. Not a round disc."""
    buf = p.canvas(width, height)
    cx, cy = width // 2, height // 2 + 1
    ry = max(height // 6, 2)
    p.ellipse(buf, width, height, cx + 1, cy, width // 2 - 1, ry, (*p.LEAF_SH[:3], 95))
    p.ellipse(buf, width, height, cx - width // 6, cy, width // 4, max(ry - 1, 2), (*p.LEAF_SH[:3], 150))
    p.ellipse(buf, width, height, cx - width // 4, cy, max(width // 8, 3), max(ry - 1, 2), (*p.LEAF_SH[:3], 185))
    return bytes(buf)


def main() -> None:
    ROOT.mkdir(parents=True, exist_ok=True)
    p.write_png(ROOT / "tree_a.png", 96, 96, oak())
    p.write_png(ROOT / "tree_b.png", 64, 96, conifer())
    p.write_png(ROOT / "tree_c.png", 64, 64, birch())
    p.write_png(ROOT / "bush_a.png", 32, 32, bush(71))
    p.write_png(ROOT / "bush_b.png", 32, 32, bush(73))
    p.write_png(ROOT / "bush_c.png", 32, 32, bush(79))
    p.write_png(ROOT / "fern_a.png", 32, 32, fern(19))
    p.write_png(ROOT / "fern_b.png", 32, 32, fern(29))
    p.write_png(ROOT / "flower_a.png", 16, 16, flower(p.PETAL_A))
    p.write_png(ROOT / "flower_b.png", 16, 16, flower(p.PETAL_B))
    p.write_png(ROOT / "mushroom_a.png", 16, 16, mushroom())
    p.write_png(ROOT / "clump_a.png", 32, 16, clump())
    p.write_png(ROOT / "grass_short.png", 16, 16, hierba(16, 16, 2, False))
    p.write_png(ROOT / "grass_short_b.png", 16, 16, hierba(16, 16, 7, False))
    p.write_png(ROOT / "grass_tall.png", 24, 32, hierba(24, 32, 3, True))
    p.write_png(ROOT / "grass_tall_b.png", 24, 32, hierba(24, 32, 11, True))
    p.write_png(ROOT / "shadow.png", 72, 20, shadow(72, 20))
    p.write_png(ROOT / "shadow_sm.png", 40, 12, shadow(40, 12))
    p.write_png(ROOT / "player_s.png", 32, 32, player("s", False))
    p.write_png(ROOT / "player_s2.png", 32, 32, player("s", True))
    p.write_png(ROOT / "player_n.png", 32, 32, player("n", False))
    p.write_png(ROOT / "player_n2.png", 32, 32, player("n", True))
    east = player("e", False)
    east2 = player("e", True)
    p.write_png(ROOT / "player_e.png", 32, 32, east)
    p.write_png(ROOT / "player_e2.png", 32, 32, east2)
    p.write_png(ROOT / "player_w.png", 32, 32, flip_h(east, 32, 32))
    p.write_png(ROOT / "player_w2.png", 32, 32, flip_h(east2, 32, 32))
    print(f"wrote props to {ROOT}")


if __name__ == "__main__":
    main()
