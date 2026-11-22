#!/usr/bin/env python3
"""Generate cosmic-blue-prompt.ttf, the few symbols of the shell prompt.

COSMIC Terminal spaces lines at 1.4 times the font size, while powerline
arrows from Nerd Fonts are drawn at the font's own height (about 1.2 em), so
filled arrows end up shorter than the colored backgrounds around them. These
glyphs are sized for COSMIC Terminal's cell with its default font, Noto Sans
Mono (advance 600, ascent 1069, descent 293 per 1000 units): the arrow spans
the whole 1.4 em line. The font also holds the cosmic-blue logo mark and the
other prompt icons, so a prompt line only uses these two fonts and keeps the
same baseline.

The glyphs sit in plane 16 private use code points, which Nerd Fonts and
other fonts leave free, so terminals always fall back to this font for them:
  U+10CB00 logo mark   U+10CB01 arrow   U+10CB02 git branch
  U+10CB03 cross       U+10CB04 lock
  U+10CB05 upper half  U+10CB06 lower half (the fastfetch logo's pixels)

Needs fontTools. Usage: python3 build.py OUTPUT.ttf
       python3 build.py --check FONT.ttf  (fails when FONT.ttf's outlines or
       metrics differ from this script's, e.g. a change here that was not
       followed by a rebuild of the committed font)
"""
import math
import sys

from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen

UPM, ADVANCE, ASCENT, DESCENT = 1000, 600, 1069, 293
# Where COSMIC Terminal puts a line's edges, at its default size of 14 px:
# lines are ceil(1.4 * size) pixels high (cosmic-term config.rs), the text is
# centered in them by its ascent and descent, and cosmic-text truncates the
# baseline to a whole pixel (layout.rs, "hinting in Y axis"). So at 14 px the
# line is 20 px, with the baseline 15 px below its top. Other sizes put the
# edges up to about half a pixel away; the default size is the one that
# matters most.
SIZE = 14
LINE_PX = math.ceil(1.4 * SIZE)
BASELINE_PX = math.floor((LINE_PX - (ASCENT + DESCENT) * SIZE / UPM) / 2
                         + ASCENT * SIZE / UPM)
# Cell edges around the baseline, a few units further out so antialiasing
# leaves no seam between lines, but too few to tint the lines around
OVERSHOOT = 4
CELL_TOP = BASELINE_PX * UPM / SIZE + OVERSHOOT
CELL_BOTTOM = -(LINE_PX - BASELINE_PX) * UPM / SIZE - OVERSHOOT
ICON_Y = 360  # vertical center of the icons: the middle of capital letters


def polygon(pen, points):
    """One contour, clockwise in font coordinates (y up), as TrueType wants."""
    area = sum(x0 * y1 - x1 * y0
               for (x0, y0), (x1, y1) in zip(points, points[1:] + points[:1]))
    if area > 0:
        points = points[::-1]
    pen.moveTo(points[0])
    for p in points[1:]:
        pen.lineTo(p)
    pen.closePath()


def arc(cx, cy, r, a0, a1, steps=48):
    return [(round(cx + r * math.cos(math.radians(a0 + (a1 - a0) * i / steps))),
             round(cy + r * math.sin(math.radians(a0 + (a1 - a0) * i / steps))))
            for i in range(steps + 1)]


def circle(pen, cx, cy, r):
    polygon(pen, arc(cx, cy, r, 0, 360)[:-1])


def stroke(pen, x0, y0, x1, y1, width):
    dx, dy = x1 - x0, y1 - y0
    n = math.hypot(dx, dy)
    ox, oy = -dy / n * width / 2, dx / n * width / 2
    polygon(pen, [(round(x0 + ox), round(y0 + oy)), (round(x1 + ox), round(y1 + oy)),
                  (round(x1 - ox), round(y1 - oy)), (round(x0 - ox), round(y0 - oy))])


def logo(pen):
    """The logo's "orbit c" (branding/logo/build.py), in one color."""
    scale, cx, cy = 8.5, ADVANCE / 2, ICON_Y

    def pt(x, y):  # logo units (y down, -50..50) to font units
        return (cx + x * scale, cy - y * scale)

    # Orbit: from -40 degrees the long way round to +40 degrees (y down),
    # stroke width 13 around radius 40, with round caps
    outer = arc(0, 0, 46.5, -40, -320)
    inner = arc(0, 0, 33.5, -320, -40)
    end = (40 * math.cos(math.radians(-320)), 40 * math.sin(math.radians(-320)))
    start = (40 * math.cos(math.radians(-40)), 40 * math.sin(math.radians(-40)))
    cap_end = arc(end[0], end[1], 6.5, -320, -320 - 180, 16)
    cap_start = arc(start[0], start[1], 6.5, -40 + 180, -40, 16)
    ring = outer + cap_end[1:-1] + inner + cap_start[1:-1]
    polygon(pen, [tuple(round(v) for v in pt(x, y)) for x, y in ring])
    for x, y, r in ((-2, 0, 13), (38, 0, 6.5)):  # planet and moon
        px, py = pt(x, y)
        circle(pen, px, py, r * scale)


# The arrow is wider than its cell, like powerline arrows in terminals with
# tighter lines: its tip reaches into the next cell, which the prompt always
# fills with a space (COSMIC Terminal draws all cell backgrounds first)
ARROW_TIP = 1000


# Its back edge starts inside the cell before, a space of the arrow's own color,
# so antialiasing leaves no seam between the segment and its arrow
ARROW_BACK = -150
# The slanted sides start at the cell edge, exactly where the segment's
# background ends: starting them at the back edge would leave a small step at
# the segment's corners. They start a little before the cell edge, so they
# cross the line's real top and bottom (the cell edges above are OVERSHOOT units
# further out) right on it
ARROW_MID = round((CELL_TOP + CELL_BOTTOM) / 2)
ARROW_START = round(-ARROW_TIP * OVERSHOOT / (CELL_TOP - ARROW_MID - OVERSHOOT))


def arrow(pen):
    polygon(pen, [(ARROW_BACK, round(CELL_TOP)),
                  (ARROW_START, round(CELL_TOP)),
                  (ARROW_TIP, ARROW_MID),
                  (ARROW_START, round(CELL_BOTTOM)),
                  (ARROW_BACK, round(CELL_BOTTOM))])


def branch(pen):
    w = 80
    stroke(pen, 170, 60, 170, 660, w)
    stroke(pen, 450, 600, 450, 450, w)
    stroke(pen, 450, 450, 170, 260, w)
    for x, y in ((170, 40), (170, 680), (450, 640)):
        circle(pen, x, y, 95)


def cross(pen):
    d = 240
    stroke(pen, 300 - d, ICON_Y - d, 300 + d, ICON_Y + d, 110)
    stroke(pen, 300 - d, ICON_Y + d, 300 + d, ICON_Y - d, 110)


def lock(pen):
    polygon(pen, [(110, 20), (110, 400), (490, 400), (490, 20)])
    polygon(pen, arc(300, 400, 160, 0, 180) + [(140, 380), (200, 380)]
            + arc(300, 400, 100, 180, 0) + [(400, 380), (460, 380)])


# Half cells for the fastfetch logo (branding/fastfetch/build.py): block
# elements from other fonts stop short of the cell bottom in COSMIC Terminal,
# these span exactly the upper or lower half of the whole line, plus a few
# units to the sides so no seam shows between cells. The two halves also
# overlap by a few units at the middle: it rarely falls on a pixel edge, and
# two antialiased edges there would leave a faint dark line inside the logo
MIDDLE = round((CELL_TOP + CELL_BOTTOM) / 2)
OVERLAP = 8


def upper_half(pen):
    polygon(pen, [(-8, round(CELL_TOP)), (ADVANCE + 8, round(CELL_TOP)),
                  (ADVANCE + 8, MIDDLE - OVERLAP), (-8, MIDDLE - OVERLAP)])


def lower_half(pen):
    polygon(pen, [(-8, MIDDLE + OVERLAP), (ADVANCE + 8, MIDDLE + OVERLAP),
                  (ADVANCE + 8, round(CELL_BOTTOM)), (-8, round(CELL_BOTTOM))])


GLYPHS = {
    ".notdef": (None, None),
    "space": (0x20, None),
    "logo": (0x10CB00, logo),
    "arrow": (0x10CB01, arrow),
    "branch": (0x10CB02, branch),
    "cross": (0x10CB03, cross),
    "lock": (0x10CB04, lock),
    "upperhalf": (0x10CB05, upper_half),
    "lowerhalf": (0x10CB06, lower_half),
}


def build():
    fb = FontBuilder(UPM, isTTF=True)
    order = list(GLYPHS)
    fb.setupGlyphOrder(order)
    fb.setupCharacterMap({cp: name for name, (cp, _) in GLYPHS.items() if cp})
    glyphs = {}
    for name, (_, draw) in GLYPHS.items():
        pen = TTGlyphPen(None)
        if draw:
            draw(pen)
        glyphs[name] = pen.glyph()
    fb.setupGlyf(glyphs)
    glyf = fb.font["glyf"]
    for name in order:
        glyf[name].recalcBounds(glyf)
    fb.setupHorizontalMetrics({name: (ADVANCE, getattr(glyf[name], "xMin", 0))
                               for name in order})
    fb.setupHorizontalHeader(ascent=ASCENT, descent=-DESCENT)
    fb.setupNameTable({"familyName": "cosmic-blue Prompt", "styleName": "Regular",
                       "uniqueFontIdentifier": "cosmic-blue Prompt Regular",
                       "fullName": "cosmic-blue Prompt Regular",
                       "psName": "cosmic-blue-Prompt-Regular",
                       "copyright": "cosmic-blue contributors, Apache-2.0"})
    fb.setupOS2(version=4, sTypoAscender=ASCENT, sTypoDescender=-DESCENT, sTypoLineGap=0,
                usWinAscent=ASCENT, usWinDescent=DESCENT, fsSelection=0x40 | 0x80,
                xAvgCharWidth=ADVANCE)
    fb.font["OS/2"].panose.bProportion = 9  # monospaced
    fb.setupPost(isFixedPitch=1)
    return fb.font


def outlines(font):
    """What the terminal draws: each glyph's points, advance and code point."""
    cmap = {name: cp for cp, name in font.getBestCmap().items()}
    glyf, hmtx = font["glyf"], font["hmtx"]
    return {name: (cmap.get(name), hmtx[name],
                   list(glyf[name].getCoordinates(glyf)[0]),
                   list(glyf[name].getCoordinates(glyf)[1]))
            for name in font.getGlyphOrder()}


def check(path):
    from fontTools.ttLib import TTFont
    font, built = TTFont(path), build()
    expected = outlines(built)
    actual = outlines(font)
    different = sorted(name for name in expected.keys() | actual.keys()
                       if expected.get(name) != actual.get(name))
    metrics = [(table, attr) for table, attr in
               (("hhea", "ascent"), ("hhea", "descent"), ("head", "unitsPerEm"))
               if getattr(font[table], attr) != getattr(built[table], attr)]
    if different or metrics:
        print(f"{path} is not what {sys.argv[0]} builds (glyphs: "
              f"{', '.join(different) or 'none'}; metrics: "
              f"{', '.join(f'{t}.{a}' for t, a in metrics) or 'none'}). "
              f"Rebuild it: python3 {sys.argv[0]} {path}", file=sys.stderr)
        sys.exit(1)
    print(f"{path} matches {sys.argv[0]}")


if __name__ == "__main__":
    if sys.argv[1] == "--check":
        check(sys.argv[2])
    else:
        build().save(sys.argv[1])
