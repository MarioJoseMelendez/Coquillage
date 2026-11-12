"""Typographic wordmark. Letters only. No mark, no frame."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import pix as p

GLYPHS = {
    "C": (" ### ", "#   #", "#    ", "#    ", "#    ", "#   #", " ### "),
    "o": ("     ", " ### ", "#   #", "#   #", "#   #", " ### ", "     "),
    "q": ("     ", " ### ", "#   #", "# # #", " ### ", "   # ", "   # "),
    "u": ("     ", "#   #", "#   #", "#   #", "#   #", " ### ", "     "),
    "i": ("  #  ", "     ", "  #  ", "  #  ", "  #  ", "  #  ", "     "),
    "l": ("  #  ", "  #  ", "  #  ", "  #  ", "  #  ", "  ## ", "     "),
    "a": ("     ", " ### ", "    #", " ####", "#   #", " ####", "     "),
    "g": ("     ", " ### ", "#   #", " ####", "#   #", " ### ", "   # "),
    "e": ("     ", " ### ", "#    ", "#### ", "#    ", " ### ", "     "),
}


def main() -> None:
    word = "Coquillage"
    scale = 6
    gap = 1
    gw, gh = 5, 7
    text_w = len(word) * (gw + gap) - gap
    sw, sh = text_w + 1, gh + 1
    buf = p.canvas(sw, sh)
    x0 = 0
    for ch in word:
        glyph = GLYPHS[ch]
        for gy, row in enumerate(glyph):
            for gx, bit in enumerate(row):
                if bit != "#":
                    continue
                p.put(buf, sw, x0 + gx + 1, gy + 1, p.LEAF_SH)
                p.put(buf, sw, x0 + gx, gy, p.LEAF_HI)
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
