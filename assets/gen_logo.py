#!/usr/bin/env python3
"""Generate the Pioneer Trail logo in every form from one pixel source.

Outputs:
  crates/data/art/title.px   in-game title art (one char per pixel, palette letters)
  assets/logo.svg            README/GitHub logo (8x pixel scale)
  assets/logo.png            via rsvg-convert if present

Palette letters and hex values are the six Apple II hi-res colors; see docs/DESIGN.md.
Run:  python3 assets/gen_logo.py
"""
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parent.parent
W, H = 80, 32

PALETTE = {
    "K": "#000000",  # black
    "W": "#FFFFFF",  # white
    "G": "#1BCB01",  # green
    "V": "#E434FE",  # violet
    "O": "#F26A00",  # orange
    "B": "#1B9AFE",  # blue
}

# --- sprites -----------------------------------------------------------------

WAGON = """
..........WWWWWWWWWWWWWWWWWWWW..........
.......WWWWWWWWWWWWWWWWWWWWWWWWWW.......
.....WWWWWWWWWWWWWWWWWWWWWWWWWWWWWW.....
....WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW....
...WWWKWWWWWKWWWWWWKWWWWWWKWWWWWKWWWW...
...WWWKWWWWWKWWWWWWKWWWWWWKWWWWWKWWWW...
...WWWKWWWWWKWWWWWWKWWWWWWKWWWWWKWWWW...
...WWWKWWWWWKWWWWWWKWWWWWWKWWWWWKWWWW...
..OOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO..
.OOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO.
.OOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO.
..OOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO..
....KKKK........................KKKK....
...KKWWKK......................KKWWKK...
..KKWKKWKK....................KKWKKWKK..
..KWKKKKWK....................KWKKKKWK..
..KWKKKKWK....................KWKKKKWK..
..KKWKKWKK....................KKWKKWKK..
...KKWWKK......................KKWWKK...
....KKKK........................KKKK....
"""

# One ox, facing left (west). Horns in white, hooves black.
OX = """
W....W..............
.W..W...............
..WOOO..OOOOOOOOOO..
..OOOOOOOOOOOOOOOOO.
..OKOOOOOOOOOOOOOOOO
..OOOOOOOOOOOOOOOOOO
..OOOOOOOOOOOOOOOOOO
...OOO.OOO....OOO.OO
...OOO.OOO....OOO.OO
...KK..KK.....KK..KK
"""

FONT = {
    "P": ["WWWW.", "W...W", "W...W", "WWWW.", "W....", "W....", "W...."],
    "I": ["WWWWW", "..W..", "..W..", "..W..", "..W..", "..W..", "WWWWW"],
    "O": [".WWW.", "W...W", "W...W", "W...W", "W...W", "W...W", ".WWW."],
    "N": ["W...W", "WW..W", "W.W.W", "W..WW", "W...W", "W...W", "W...W"],
    "E": ["WWWWW", "W....", "W....", "WWWW.", "W....", "W....", "WWWWW"],
    "R": ["WWWW.", "W...W", "W...W", "WWWW.", "W.W..", "W..W.", "W...W"],
    "T": ["WWWWW", "..W..", "..W..", "..W..", "..W..", "..W..", "..W.."],
    "A": [".WWW.", "W...W", "W...W", "WWWWW", "W...W", "W...W", "W...W"],
    "L": ["W....", "W....", "W....", "W....", "W....", "W....", "WWWWW"],
    " ": [".....", ".....", ".....", ".....", ".....", ".....", "....."],
}


def sprite(s: str) -> list[str]:
    return [line for line in s.strip("\n").splitlines()]


def blit(canvas, art, x0, y0, recolor=None):
    for dy, row in enumerate(art):
        for dx, ch in enumerate(row):
            if ch == ".":
                continue
            if recolor and ch in recolor:
                ch = recolor[ch]
            x, y = x0 + dx, y0 + dy
            if 0 <= x < W and 0 <= y < H:
                canvas[y][x] = ch


def text(canvas, s, x0, y0, color="O"):
    x = x0
    for ch in s:
        glyph = FONT[ch]
        blit(canvas, glyph, x, y0, recolor={"W": color})
        x += 6


def build() -> list[list[str]]:
    c = [["." for _ in range(W)] for _ in range(H)]
    # ground
    for x in range(W):
        c[20][x] = "G"
        c[21][x] = "G" if x % 3 else "K"
    # oxen and tongue
    blit(c, sprite(OX), 0, 10)
    blit(c, sprite(OX), 20, 10)
    for x in range(36, 42):
        c[14][x] = "O"
    # wagon
    blit(c, sprite(WAGON), 40, 0)
    # title
    text(c, "PIONEER TRAIL", 1, 24, color="O")
    return c


def write_px(c):
    out = ROOT / "crates/data/art/title.px"
    out.write_text("\n".join("".join(r) for r in c) + "\n")
    return out


def write_svg(c, scale=8):
    out = ROOT / "assets/logo.svg"
    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{W*scale}" height="{H*scale}" '
        f'viewBox="0 0 {W} {H}" shape-rendering="crispEdges">',
        f'<rect width="{W}" height="{H}" fill="#000000"/>',
    ]
    # merge horizontal runs into one rect each to keep the file small
    for y, row in enumerate(c):
        x = 0
        while x < W:
            ch = row[x]
            if ch == ".":
                x += 1
                continue
            run = 1
            while x + run < W and row[x + run] == ch:
                run += 1
            parts.append(f'<rect x="{x}" y="{y}" width="{run}" height="1" fill="{PALETTE[ch]}"/>')
            x += run
    parts.append("</svg>")
    out.write_text("\n".join(parts) + "\n")
    return out


def write_png(svg: Path):
    if not shutil.which("rsvg-convert"):
        return None
    out = ROOT / "assets/logo.png"
    subprocess.run(["rsvg-convert", "-w", "1280", str(svg), "-o", str(out)], check=True)
    return out


def preview(c):
    """Half-block terminal preview, same technique the game renderer uses."""
    ansi = {"K": (0, 0, 0), "W": (255, 255, 255), "G": (27, 203, 1), "V": (228, 52, 254),
            "O": (242, 106, 0), "B": (27, 154, 254), ".": (0, 0, 0)}
    for y in range(0, H, 2):
        line = ""
        for x in range(W):
            t, b = ansi[c[y][x]], ansi[c[y + 1][x]]
            line += f"\x1b[38;2;{t[0]};{t[1]};{t[2]}m\x1b[48;2;{b[0]};{b[1]};{b[2]}m▀"
        print(line + "\x1b[0m")


if __name__ == "__main__":
    c = build()
    px = write_px(c)
    svg = write_svg(c)
    png = write_png(svg)
    print(f"wrote {px.relative_to(ROOT)}, {svg.relative_to(ROOT)}"
          + (f", {png.relative_to(ROOT)}" if png else ""))
    preview(c)
