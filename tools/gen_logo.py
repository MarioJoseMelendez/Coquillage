"""Pixel wordmark for the README. 5×7 glyphs, nearest scale, game palette."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import pix as p

# Rows are top to bottom. '#' is ink. q and g keep a short tail inside the cell.
GLYPHS = {
    "C": (
        ".###.",
        "#...#",
        "#....",
        "#....",
        "#....",
        "#...#",
        ".###.",
    ),
    "o": (
        ".....",
        ".###.",
        "#...#",
        "#...#",
        "#...#",
        ".###.",
        ".....",
    ),
    "q": (
        ".....",
        ".###.",
        "#...#",
        "#.#.#",
        ".###.",
        "...#.",
        "...#.",
    ),
    "u": (
        ".....",
        "#...#",
        "#...#",
        "#...#",
        "#...#",
        ".###.",
        ".....",
    ),
    "i": (
        "..#..",
        ".....",
        "..#..",
        "..#..",
        "..#..",
        "..#..",
        ".....",
    ),
    "l": (
        "..#..",
        "..#..",
        "..#..",
        "..#..",
        "..#..",
        "..##.",
        ".....",
    ),
    "a": (
        ".....",
        ".###.",
        "....#",
        ".####",
        "#...#",
        ".####",
        ".....",
    ),
    "g": (
        ".....",
        ".###.",
        "#...#",
        ".####",
        "#...#",
        ".###.",
        "...#.",
    ),
    "e": (
        ".....",
        ".###.",
        "#....",
        "####.",
        "#....",
        ".###.",
        ".....",
    ),
}


def shell(buf, w: int, ox: int, oy: int) -> None:
    """Tiny top-down shell. Open spiral, not a disc and not a plus."""
    marks = (
        (3, 0, p.SAND_HI),
        (4, 1, p.SAND),
        (4, 2, p.SAND_DK),
        (3, 3, p.SAND_DK),
        (2, 3, p.BARK),
        (1, 2, p.SAND),
        (1, 1, p.SAND_HI),
        (2, 1, p.BIRCH),
        (3, 1, p.SAND),
        (3, 2, p.SAND_HI),
        (2, 2, p.LEAF_SH),
    )
    for dx, dy, col in marks:
        p.put(buf, w, ox + dx, oy + dy, col)


def main() -> None:
    word = "Coquillage"
    scale = 6
    gap = 1
    gw, gh = 5, 7
    pad = 4
    text_w = len(word) * (gw + gap) - gap
    shell_w = 8
    sw = pad + shell_w + 3 + text_w + pad
    sh = pad + gh + pad
    buf = p.canvas(sw, sh)
    p.fill(buf, sw, sh, p.LEAF_SH)
    for y in range(sh):
        p.put(buf, sw, 0, y, p.SAND_DK)
        p.put(buf, sw, sw - 1, y, p.SAND_DK)
    for x in range(sw):
        p.put(buf, sw, x, 0, p.SAND_DK)
        p.put(buf, sw, x, sh - 1, p.SAND_DK)
    shell(buf, sw, pad, pad + 1)
    x0 = pad + shell_w + 3
    for ch in word:
        glyph = GLYPHS[ch]
        for gy, row in enumerate(glyph):
            for gx, bit in enumerate(row):
                if bit != "#":
                    continue
                p.put(buf, sw, x0 + gx + 1, pad + gy + 1, p.WATER_SH)
                p.put(buf, sw, x0 + gx, pad + gy, p.LEAF_HI)
        x0 += gw + gap
    dw, dh = sw * scale, sh * scale
    big = p.canvas(dw, dh)
    for y in range(sh):
        for x in range(sw):
            col = tuple(buf[(y * sw + x) * 4 : (y * sw + x) * 4 + 4])
            for dy in range(scale):
                for dx in range(scale):
                    p.put(big, dw, x * scale + dx, y * scale + dy, col)
    out = Path(__file__).resolve().parents[1] / "assets" / "brand" / "coquillage-logo.png"
    out.parent.mkdir(parents=True, exist_ok=True)
    p.write_png(out, dw, dh, bytes(big))
    print(out, dw, dh)


if __name__ == "__main__":
    main()
