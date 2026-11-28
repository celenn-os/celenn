#!/usr/bin/env python3
"""Generate the Celenn OS logo SVGs.

The wordmark uses Montserrat Light outlines (glyphs.json: one path per glyph
plus its horizontal extent), so the SVGs need no font at runtime.
Usage: python3 build.py  (writes the SVGs next to this script)
"""
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
J = json.loads((HERE / "glyphs.json").read_text())
# the wordmark "celenn", and the full name "Celenn OS" (wallpaper, boot screen)
WORD, NAME = (J["gl"], J["bb"]), (J["name"]["gl"], J["name"]["bb"])
TOP, BASE = 2607.672, 2670.336  # ascender top and baseline of the outlines

DEFS = """<defs>
<linearGradient id="cb-orbit" x1="0" y1="1" x2="1" y2="0"><stop offset="0" stop-color="#7fe0ff"/><stop offset="1" stop-color="#2f6bff"/></linearGradient>
<linearGradient id="cb-planet" x1=".2" y1="1" x2=".8" y2="0"><stop offset="0" stop-color="#cfeeff"/><stop offset=".45" stop-color="#4f98f5"/><stop offset="1" stop-color="#14399e"/></linearGradient>
</defs>"""


def mark(moon="#e6f3ff"):
    """The "orbit c": an open orbit around a planet, with a moon in the gap."""
    return (
        '<path d="M30.3 -25.4 A40 40 0 1 0 30.3 25.4" fill="none" stroke="url(#cb-orbit)" '
        'stroke-width="13" stroke-linecap="round"/>'
        '<circle cx="-2" cy="0" r="13" fill="url(#cb-planet)"/>'
        f'<circle cx="38" cy="0" r="6.5" fill="{moon}"/>'
    )


def wordmark(fill, letters=WORD, tracking=12):
    parts, x = [], 0.0
    for glyph, (x0, x1) in zip(*letters):
        if glyph:  # a space has no outline
            parts.append(f'<path transform="translate({x - x0:.2f} 0)" d="{glyph}"/>')
        x += (x1 - x0) + tracking
    return f'<g fill="{fill}">{"".join(parts)}</g>', x - tracking


def svg(viewbox, body, title="Celenn OS"):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{viewbox}">'
            f"<title>{title}</title>{DEFS}{body}</svg>\n")


def horizontal(fill, moon, letters=WORD):
    text, width = wordmark(fill, letters)
    height = 54.0                      # wordmark height, in mark units
    scale = height / (BASE - TOP)
    body = (f'<g transform="translate(50 50)">{mark(moon)}</g>'
            f'<g transform="translate(118 {50 - height / 2 - 4:.2f}) scale({scale:.5f}) '
            f'translate(0 {-TOP})">{text}</g>')
    return svg(f"0 0 {118 + width * scale + 6:.1f} 100", body)


(HERE / "celenn-logo.svg").write_text(svg("-50 -50 100 100", mark()))
# One-color mark for symbolic uses (the Celenn OS group of the App Library);
# the desktop recolors symbolic icons, #bebebe is only the convention
(HERE / "celenn-logo-symbolic.svg").write_text(
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="-50 -50 100 100"><title>Celenn OS</title>'
    '<g fill="#bebebe"><path d="M30.3 -25.4 A40 40 0 1 0 30.3 25.4" fill="none" stroke="#bebebe" '
    'stroke-width="13" stroke-linecap="round"/><circle cx="-2" cy="0" r="13"/>'
    '<circle cx="38" cy="0" r="6.5"/></g></svg>\n')
(HERE / "celenn-logo-horizontal-dark.svg").write_text(horizontal("#e6f3ff", "#e6f3ff"))
(HERE / "celenn-logo-horizontal-light.svg").write_text(horizontal("#0b2a8a", "#2f6bff"))
# Mark + "Celenn OS", for the default wallpaper and the boot screen
(HERE / "celenn-os-logo-horizontal-dark.svg").write_text(horizontal("#e6f3ff", "#e6f3ff", NAME))
