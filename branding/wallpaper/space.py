#!/usr/bin/env python3
"""Space telescope wallpapers: Webb and Hubble images (NASA, ESA, CSA,
STScI; CC BY 4.0), cropped to 16:9 at 5120x2880 and recoloured in the
cosmic-blue palette: warm tones turned to blues, brighter mid-tones.
Usage: space.py NAME IN.jpg OUT.jpg [SCALE]
Needs numpy and pillow.
"""
import sys

import numpy as np
from PIL import Image

Image.MAX_IMAGE_PIXELS = None

# name: (source id, crop centre x, y, crop width as a fraction of the image,
#        hue spread kept around blue (0 = all blue, 1 = original hues),
#        gradient-map strength, gamma, exposure)
IMAGES = {
    "cosmic-cliffs": ("weic2205a", .50, .50, 1.00, .12, .30, .80, 1.10),
    "tarantula": ("weic2212a", .52, .50, 1.00, .12, .30, .75, 1.20),
    "westerlund": ("heic1509a", .50, .45, 1.00, .12, .30, .80, 1.15),
    "snow-angel": ("heic1118a", .50, .52, 1.00, .12, .30, .80, 1.10),
}

name, src, out = sys.argv[1:4]
K = float(sys.argv[4]) if len(sys.argv) > 4 else 1.0
_, cx, cy, cw, spread, strength, gamma, expo = IMAGES[name]
im = Image.open(src).convert("RGB")
w, h = im.size
cwp = min(w * cw, h * 16 / 9)
chp = cwp * 9 / 16
x0 = min(max(cx * w - cwp / 2, 0), w - cwp)
y0 = min(max(cy * h - chp / 2, 0), h - chp)
W, H = int(5120 * K), int(2880 * K)
im = im.crop((int(x0), int(y0), int(x0 + cwp), int(y0 + chp))).resize((W, H), Image.LANCZOS)

# pull every hue toward the blue of the logo, keeping a little of the
# spread (warm tones end up azure or cyan); the telescope colours are
# false colours anyway
hsv = np.asarray(im.convert("HSV")).astype(np.float32)
blue = 220 / 360 * 256
d = (hsv[..., 0] - blue + 128) % 256 - 128
hsv[..., 0] = (blue - np.abs(d) * spread) % 256
a = np.asarray(Image.fromarray(hsv.astype(np.uint8), "HSV").convert("RGB")).astype(np.float32) / 255
a = np.clip(a * expo, 0, 1) ** gamma
lum = np.clip((a * [.2126, .7152, .0722]).sum(-1), 0, 1)
stops = np.array([[6, 14, 48], [29, 71, 196], [79, 141, 255], [176, 214, 255], [246, 250, 255]], np.float32) / 255
pos = [0, .25, .5, .75, 1.0]
g = np.stack([np.interp(lum, pos, stops[:, i]) for i in range(3)], -1)
img = np.clip(a * (1 - strength) + g * strength, 0, 1) * 255
img += np.random.default_rng(1).normal(0, 1.0, img.shape[:2])[..., None]
Image.fromarray(np.clip(img, 0, 255).astype(np.uint8)).save(out, quality=90, subsampling=0, optimize=True)
