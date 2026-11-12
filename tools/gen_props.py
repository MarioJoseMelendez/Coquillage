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


def stamp_body(buf: bytearray, w: int, cx: int, cy: int) -> None:
    """Shoulders wider than the hem. Seen from above, not a front-facing torso."""
    spans = ((-4, 4), (-5, 5), (-5, 5), (-4, 4), (-3, 3), (-2, 2))
    for i, (x0, x1) in enumerate(spans):
        y = cy - 2 + i
        for x in range(cx + x0, cx + x1 + 1):
            if x >= cx + 3:
                col = p.SHIRT_DK
            elif x == cx - 3 and i == 1:
                col = p.SHIRT_HI
            else:
                col = p.SHIRT
            p.put(buf, w, x, y, col)


def stamp_head(buf: bytearray, w: int, cx: int, cy: int, facing: str) -> None:
    """Scalp only. Corners stay empty so it is not a disc. No face."""
    if facing == "n":
        pixels = (
            (-2, 0, p.BARK_DK), (-1, 0, p.BARK), (0, 0, p.BARK), (1, 0, p.BARK), (2, 0, p.BARK_HI),
            (-2, 1, p.BARK), (-1, 1, p.BARK), (0, 1, p.BARK_DK), (1, 1, p.BARK), (2, 1, p.BARK),
            (-1, 2, p.BARK_DK), (0, 2, p.BARK), (1, 2, p.BARK),
        )
    elif facing == "e":
        pixels = (
            (0, 0, p.SAND_HI), (1, 0, p.SAND), (2, 0, p.SAND_HI),
            (-1, 1, p.BARK), (0, 1, p.SAND), (1, 1, p.SAND_HI), (2, 1, p.SAND),
            (0, 2, p.SAND_DK), (1, 2, p.SAND),
        )
    else:
        pixels = (
            (-1, 0, p.SAND_HI), (0, 0, p.SAND_HI), (1, 0, p.SAND),
            (-2, 1, p.SAND), (-1, 1, p.SAND_HI), (0, 1, p.SAND), (1, 1, p.SAND), (2, 1, p.SAND_DK),
            (-1, 2, p.SAND), (0, 2, p.SAND_DK), (1, 2, p.SAND),
        )
    for dx, dy, col in pixels:
        p.put(buf, w, cx + dx, cy + dy, col)


def player(facing: str, step: bool) -> bytes:
    """90° only. Scalp, shoulders, footprint. No face, no cloak ellipse.

    The old phone sprite is a sand cap over a blue body. This is that read,
    with the game's cluster shading. W is a flip of E.
    """
    w = h = 32
    buf = p.canvas(w, h)
    bob = 1 if step else 0
    if facing == "n":
        stamp_body(buf, w, 16, 18 - bob)
        stamp_head(buf, w, 16, 11, "n")
    elif facing == "s":
        stamp_body(buf, w, 16, 18 + bob)
        stamp_head(buf, w, 16, 10, "s")
        p.put(buf, w, 15, 23 + bob, p.SAND_DK)
        p.put(buf, w, 17, 23 + bob, p.SAND_DK)
    else:
        stamp_body(buf, w, 16, 18)
        stamp_head(buf, w, 18, 10, "e")
        p.put(buf, w, 18, 23, p.SAND_DK)
    buf = outline(buf, w, h, p.LEAF_SH)
    stamp_hands(buf, w, facing, step)
    return bytes(buf)


def stamp_hands(buf: bytearray, w: int, facing: str, step: bool) -> None:
    """2×2 flats beside the shoulders. One dark corner, not a disc."""
    bob = 1 if step else 0

    def flat(x: int, y: int) -> None:
        p.put(buf, w, x, y, p.SAND_HI)
        p.put(buf, w, x + 1, y, p.SAND)
        p.put(buf, w, x, y + 1, p.SAND)
        p.put(buf, w, x + 1, y + 1, p.SAND_DK)

    if facing == "s":
        flat(9, 17 + bob)
        flat(20, 16 + bob)
    elif facing == "n":
        flat(9, 15 - bob)
        flat(20, 16 - bob)
    else:
        flat(21, 16 + bob)
        flat(9, 18 - bob)


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
