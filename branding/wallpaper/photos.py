#!/usr/bin/env python3
"""Bright photo wallpapers from pictures taken by astronauts on the
International Space Station or the Space Shuttle (NASA, public domain),
cropped to 16:9 at 5120x2880 and graded toward the cosmic-blue palette.
Usage: photos.py NAME IN.jpg OUT.jpg [SCALE]
Needs numpy and pillow.
"""
import sys

import numpy as np
from PIL import Image, ImageFilter

Image.MAX_IMAGE_PIXELS = None

# name: (NASA id, crop centre x, crop centre y, crop width as a fraction of
#        the photo, grading strength, exposure, gamma, denoise radius)
PHOTOS = {
    "shallows": ("iss039e012272", .50, .50, 1.00, .30, 1.10, .95, .6),
    "full-moon": ("iss064e048273", .50, .58, 1.00, .25, 1.15, .90, 1.0),
    "clouds": ("iss036e025673", .50, .45, 1.00, .35, 1.05, 1.0, .5),
    "blue-planet": ("iss038e047389", .52, .50, 1.00, .30, 1.10, .95, .6),
    "sunset-clouds": ("s114e5496", .50, .55, 1.00, .40, 1.20, .90, .5),
}

name, src, out = sys.argv[1:4]
K = float(sys.argv[4]) if len(sys.argv) > 4 else 1.0
_, cx, cy, cw, strength, expo, gamma, dn = PHOTOS[name]
im = Image.open(src).convert("RGB")
w, h = im.size
cwp = min(w * cw, h * 16 / 9)
chp = cwp * 9 / 16
x0 = min(max(cx * w - cwp / 2, 0), w - cwp)
y0 = min(max(cy * h - chp / 2, 0), h - chp)
im = im.crop((int(x0), int(y0), int(x0 + cwp), int(y0 + chp)))
if dn:
    im = im.filter(ImageFilter.GaussianBlur(dn))      # soften sensor noise
W, H = int(5120 * K), int(2880 * K)
im = im.resize((W, H), Image.LANCZOS)
a = np.asarray(im).astype(np.float32) / 255
a = np.clip(a * expo, 0, 1) ** gamma
lum = np.clip((a * [.2126, .7152, .0722]).sum(-1), 0, 1)
# gradient map: deep navy, the logo blues, ice white
stops = np.array([[4, 10, 36], [29, 71, 196], [79, 141, 255], [170, 210, 255], [240, 248, 255]], np.float32) / 255
pos = [0, .25, .5, .75, 1.0]
g = np.stack([np.interp(lum, pos, stops[:, i]) for i in range(3)], -1)
# pull every hue toward blue but keep the photo's own contrast
img = a * (1 - strength) + g * strength
img = np.clip(img, 0, 1) * 255
img += np.random.default_rng(1).normal(0, 1.2, img.shape[:2])[..., None]
Image.fromarray(np.clip(img, 0, 255).astype(np.uint8)).save(out, quality=90, subsampling=0, optimize=True)
