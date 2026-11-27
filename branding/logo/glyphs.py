#!/usr/bin/env python3
"""Write glyphs.json: the outlines of the wordmark "celenn" in Montserrat
Light (weight 300), one SVG path per letter plus its ink extent, in the units
build.py expects (84 px letters, baseline at y 2670).

Needs fontTools and Montserrat's variable font from Google Fonts:
  curl -fsSLo Montserrat.ttf 'https://raw.githubusercontent.com/google/fonts/main/ofl/montserrat/Montserrat%5Bwght%5D.ttf'
  python3 glyphs.py Montserrat.ttf
"""
import json
import sys
from pathlib import Path

from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

WORD = "celenn"
K, BASE = 0.084, 2670.0  # font units to wordmark units, baseline

font = instantiateVariableFont(TTFont(sys.argv[1]), {"wght": 300})
glyphs, cmap = font.getGlyphSet(), font.getBestCmap()
paths, extents = [], []
for letter in WORD:
    glyph = glyphs[cmap[ord(letter)]]
    pen = SVGPathPen(glyphs, ntos=lambda v: f"{round(v, 3):g}")
    glyph.draw(TransformPen(pen, (K, 0, 0, -K, 0, BASE)))
    bounds = BoundsPen(glyphs)
    glyph.draw(bounds)
    paths.append(pen.getCommands())
    extents.append([round(bounds.bounds[0] * K, 3), round(bounds.bounds[2] * K, 3)])
(Path(__file__).parent / "glyphs.json").write_text(
    json.dumps({"gl": paths, "bb": extents}, separators=(",", ":")))
