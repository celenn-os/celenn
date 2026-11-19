#!/usr/bin/env python3
"""Write the official horizontal logo (orbit, planet, moon and the full
name) over a rendered background, with a soft blue glow.
Usage: logo_name.py BACKGROUND.png OUT.jpg
Needs numpy, pillow and scipy, and node with playwright (render.js).
The logo's orbit centre must match LOGO in shaders/echoes.frag.
"""
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
inner = logo[logo.index(">", logo.index("<svg")) + 1:logo.rindex("</svg>")]
W, H = 5120, 2880
k = 2900 / 559.7                    # logo width 2900 px
x0, y0 = W / 2 - 559.7 * k / 2, H / 2 + 60 - 50 * k
svg = (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}">'
       f'<g transform="translate({x0:.1f} {y0:.1f}) scale({k:.4f})">{inner}</g></svg>')
with tempfile.TemporaryDirectory() as tmp:
    (Path(tmp) / "logo.svg").write_text(svg)
    subprocess.run(["node", str(here / "render.js"), f"{tmp}/logo.svg", f"{tmp}/logo.png"], check=True)
    fg = np.asarray(Image.open(f"{tmp}/logo.png").convert("RGBA")).astype(np.float32) / 255
bg = np.asarray(Image.open(bg_path).convert("RGB")).astype(np.float32) / 255
a = fg[..., 3:4]
glow = gaussian_filter(a[..., 0], 70)[..., None]
img = bg + glow * np.array([79, 141, 255], np.float32) / 255 * .45
img = img * (1 - a) + fg[..., :3] * a
img = np.clip(img, 0, 1) * 255
img += np.random.default_rng(1).normal(0, .8, img.shape[:2])[..., None]
Image.fromarray(np.clip(img, 0, 255).astype(np.uint8)).save(out, quality=90, optimize=True)
