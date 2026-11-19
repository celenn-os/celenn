#!/usr/bin/env python3
"""Write the cosmic-blue wordmark over a rendered background, with a soft
blue glow: the "constellation" wallpaper with the full name.
Usage: name.py BACKGROUND.png OUT.jpg
Needs numpy, pillow and scipy, and node with playwright (render.js).
"""
import re
import subprocess
import sys
import tempfile
from pathlib import Path

import numpy as np
from PIL import Image
from scipy.ndimage import gaussian_filter

here = Path(__file__).parent
bg_path, out = sys.argv[1:3]
logo = (here / "../logo/cosmic-blue-logo-horizontal-dark.svg").read_text()
# the wordmark alone: the second group of the horizontal logo, which spans
# x 118..559.7 and y 19..81 in the logo's units
text = logo[logo.index('<g transform="translate(118'):logo.rindex("</svg>")]
W, H = 5120, 2880
width = 2500                       # wordmark width in pixels
k = width / (559.7 - 118)
x0 = W / 2 - width / 2 - 118 * k
y0 = H / 2 + 150 - 50 * k          # text centre a little below the middle
svg = (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}">'
       f'<g transform="translate({x0:.1f} {y0:.1f}) scale({k:.4f})">{text}</g></svg>')
with tempfile.TemporaryDirectory() as tmp:
    (Path(tmp) / "name.svg").write_text(svg)
    subprocess.run(["node", str(here / "render.js"), f"{tmp}/name.svg", f"{tmp}/name.png"], check=True)
    fg = np.asarray(Image.open(f"{tmp}/name.png").convert("RGBA")).astype(np.float32) / 255
bg = np.asarray(Image.open(bg_path).convert("RGB")).astype(np.float32) / 255
a = fg[..., 3:4]
glow = gaussian_filter(a[..., 0], 60)[..., None]
img = bg + glow * np.array([79, 141, 255], np.float32) / 255 * .35
img = img * (1 - a * .94) + np.array([230, 243, 255], np.float32) / 255 * a * .94
img = np.clip(img, 0, 1) * 255
img += np.random.default_rng(1).normal(0, .8, img.shape[:2])[..., None]
Image.fromarray(np.clip(img, 0, 255).astype(np.uint8)).save(out, quality=90, optimize=True)
