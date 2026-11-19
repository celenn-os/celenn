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
MARK = (2160, 1380)
SCALE = 9.0


def defines():
    """Shader defines that centre the echoes on the mark."""
    x, y = (MARK[0] - W / 2) / H, (H / 2 - MARK[1]) / H
    return f"#define LOGO vec2({x:.4f}, {y:.4f})\n#define R {40 * SCALE / H:.4f}"


def main():
    here = Path(__file__).parent
    bg_path, out = sys.argv[1:3]
    text_mode = sys.argv[3] if len(sys.argv) > 3 else "none"
    logo = (here / "../logo/cosmic-blue-logo-horizontal-dark.svg").read_text()
    defs = logo[logo.index("<defs>"):logo.index("</defs>") + 7]
    icon = logo[logo.index('<g transform="translate(50 50)">') + 32:logo.index('</g><g transform="translate(118')]
    text = logo[logo.index('<g transform="translate(118'):logo.rindex("</svg>")]
    parts = [f'<g transform="translate({MARK[0]} {MARK[1]}) scale({SCALE})">{icon}</g>']
    if text_mode != "none":
        width = 760 if text_mode == "corner" else 1150
        k = width / (559.7 - 118)
        if text_mode == "corner":   # bottom right, clear of the dock
            x0, y0 = W - 230 - width - 118 * k, H - 250 - 81 * k
        else:                       # centred under the mark
            x0, y0 = MARK[0] - width / 2 - 118 * k, MARK[1] + 47 * SCALE + 170 - 19 * k
        parts.append(f'<g transform="translate({x0:.1f} {y0:.1f}) scale({k:.4f})">{text}</g>')
    svg = (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}">{defs}'
           + "".join(parts) + "</svg>")
    with tempfile.TemporaryDirectory() as tmp:
        (Path(tmp) / "mark.svg").write_text(svg)
        subprocess.run(["node", str(here / "render.js"), f"{tmp}/mark.svg", f"{tmp}/mark.png"], check=True)
        fg = np.asarray(Image.open(f"{tmp}/mark.png").convert("RGBA")).astype(np.float32) / 255
    bg = np.asarray(Image.open(bg_path).convert("RGB")).astype(np.float32) / 255
    a = fg[..., 3:4]
    glow = gaussian_filter(a[..., 0], 90)[..., None]
    img = bg + glow * np.array([79, 141, 255], np.float32) / 255 * .5
    img = img * (1 - a) + fg[..., :3] * a
    img = np.clip(img, 0, 1) * 255
    img += np.random.default_rng(1).normal(0, .8, img.shape[:2])[..., None]
    Image.fromarray(np.clip(img, 0, 255).astype(np.uint8)).save(out, quality=90, optimize=True)


if __name__ == "__main__":
    if sys.argv[1:] == ["--defines"]:
        print(defines())
    else:
        main()
