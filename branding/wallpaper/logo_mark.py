#!/usr/bin/env python3
"""Draw the logo mark (orbit, planet, moon) and optionally the name over a
rendered background, with a soft blue glow.
Usage: logo_mark.py BACKGROUND.png OUT.jpg [none|corner|below]
The mark sits at MARK (pixels), its orbit radius is 40 * SCALE pixels: keep
LOGO and R of shaders/echoes.frag in step (see defines()).
Needs numpy, pillow and scipy, and node with playwright (render.js).
"""
import subprocess
import sys
import tempfile
from pathlib import Path

import numpy as np
from PIL import Image
from scipy.ndimage import gaussian_filter

W, H = 5120, 2880
# where the mark sat in the horizontal logo, centred 60 px below the middle
SCALE = 2900 / 559.7
MARK = (W / 2 - 559.7 * SCALE / 2 + 50 * SCALE, H / 2 + 60)
# COSMIC scales the wallpaper to fill the screen and crops the rest. The name
# sits close to the bottom right corner, just above the dock, as far right as
# it can while staying whole on these screen shapes (4:3, 5:4 and 21:9
# screens cut part of it).
RATIOS = {"16:9": 16 / 9, "16:10": 16 / 10, "3:2": 3 / 2}
BOTTOM = 0.075 * H  # from the bottom edge
RIGHT = 20  # from the right edge of the narrowest shape (3:2)


def visible(ratio):
    """The part of the wallpaper (x0, y0, x1, y1) a screen of this ratio shows."""
    w, h = min(W, H * ratio), min(H, W / ratio)
    return (W - w) / 2, (H - h) / 2, (W + w) / 2, (H + h) / 2


def safe_box():
    """The part every ratio in RATIOS shows."""
    boxes = [visible(r) for r in RATIOS.values()]
    return (max(b[0] for b in boxes), max(b[1] for b in boxes),
            min(b[2] for b in boxes), min(b[3] for b in boxes))


# the name (the horizontal logo's text runs from x 118 to 559.7 and from y 19
# to 81 in logo units)
NAME_WIDTH = 760
NAME_K = NAME_WIDTH / (559.7 - 118)
_right, _bottom = safe_box()[2] - RIGHT, H - BOTTOM
NAME_BOX = (_right - NAME_WIDTH, _bottom - 62 * NAME_K, _right, _bottom)


def defines():
    """Shader defines that centre the echoes on the mark."""
    x, y = (MARK[0] - W / 2) / H, (H / 2 - MARK[1]) / H
    # no stars right around the name, where one would read as a full stop
    pad = 60
    x0, y0, x1, y1 = NAME_BOX[0] - pad, NAME_BOX[1] - pad, NAME_BOX[2] + pad, NAME_BOX[3] + pad
    clear = [(x0 - W / 2) / H, (H / 2 - y1) / H, (x1 - W / 2) / H, (H / 2 - y0) / H]
    return (f"#define LOGO vec2({x:.4f}, {y:.4f})\n#define R {40 * SCALE / H:.4f}\n"
            f"#define CLEAR vec4({', '.join(f'{v:.4f}' for v in clear)})")


def main():
    here = Path(__file__).parent
    bg_path, out = sys.argv[1:3]
    text_mode = sys.argv[3] if len(sys.argv) > 3 else "none"
    logo = (here / "../logo/cosmic-blue-logo-horizontal-dark.svg").read_text()
    defs = logo[logo.index("<defs>"):logo.index("</defs>") + 7]
    icon = logo[logo.index('<g transform="translate(50 50)">') + 32:logo.index('</g><g transform="translate(118')]
    text = logo[logo.index('<g transform="translate(118'):logo.rindex("</svg>")]
    parts = [f'<g transform="translate({MARK[0]:.1f} {MARK[1]:.1f}) scale({SCALE:.4f})">{icon}</g>']
    if text_mode != "none":
        if text_mode == "corner":   # bottom right, clear of the dock
            k = NAME_K
            x0, y0 = NAME_BOX[0] - 118 * k, NAME_BOX[1] - 19 * k
        else:                       # centred under the mark
            width = 1150
            k = width / (559.7 - 118)
            x0, y0 = MARK[0] - width / 2 - 118 * k, MARK[1] + 47 * SCALE + 170 - 19 * k
        parts.append(f'<g transform="translate({x0:.1f} {y0:.1f}) scale({k:.4f})">{text}</g>')
    svg = (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}">{defs}'
           + "".join(parts) + "</svg>")
    with tempfile.TemporaryDirectory() as tmp:
        (Path(tmp) / "mark.svg").write_text(svg)
        subprocess.run(["node", str(here / "render.js"), f"{tmp}/mark.svg", f"{tmp}/mark.png"], check=True)
        fg = np.asarray(Image.open(f"{tmp}/mark.png").convert("RGBA")).astype(np.float32) / 255
    if text_mode == "corner":
        ys, xs = np.nonzero(fg[H // 2:, W // 2:, 3] > .02)
        x0, y0, x1, y1 = safe_box()
        if xs.max() + W // 2 > x1 or ys.max() + H // 2 > y1:
            sys.exit(f"the name leaves the safe box {safe_box()}")
    bg = np.asarray(Image.open(bg_path).convert("RGB")).astype(np.float32) / 255
    a = fg[..., 3:4]
    glow = gaussian_filter(a[..., 0], 70)[..., None]
    img = bg + glow * np.array([79, 141, 255], np.float32) / 255 * .45
    img = img * (1 - a) + fg[..., :3] * a
    img = np.clip(img, 0, 1) * 255
    img += np.random.default_rng(1).normal(0, .8, img.shape[:2])[..., None]
    Image.fromarray(np.clip(img, 0, 255).astype(np.uint8)).save(out, quality=90, optimize=True)


if __name__ == "__main__":
    if sys.argv[1:] == ["--defines"]:
        print(defines())
    else:
        main()
