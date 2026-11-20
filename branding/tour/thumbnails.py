#!/usr/bin/python3
"""Small copies of the wallpapers, for the first steps tour's "Make it yours"
page (build_files/first_steps): decoding the 5K originals would make the
page slow and heavy. 224 x 126 (shown at 92 x 52, sharp on HiDPI screens).

Written to system_files/usr/share/cosmic-blue/wallpapers, one per wallpaper
of system_files/usr/share/backgrounds/cosmic-blue, same file name; the image
build test checks that every wallpaper has one. Run from the repository root
after adding or changing a wallpaper: python3 branding/tour/thumbnails.py
(needs Pillow).
"""

import pathlib

from PIL import Image

SOURCE = pathlib.Path("system_files/usr/share/backgrounds/cosmic-blue")
OUT = pathlib.Path("system_files/usr/share/cosmic-blue/wallpapers")
SIZE = (224, 126)

OUT.mkdir(parents=True, exist_ok=True)
for old in OUT.glob("*.jpg"):
    if not (SOURCE / old.name).exists():
        old.unlink()
for wallpaper in sorted(SOURCE.glob("*.jpg")):
    with Image.open(wallpaper) as image:
        image = image.convert("RGB")
        # Zoomed to fill, like the desktop does (scaling_mode: Zoom)
        ratio = max(SIZE[0] / image.width, SIZE[1] / image.height)
        image = image.resize(
            (round(image.width * ratio), round(image.height * ratio)),
            Image.Resampling.LANCZOS,
        )
        left = (image.width - SIZE[0]) // 2
        top = (image.height - SIZE[1]) // 2
        image = image.crop((left, top, left + SIZE[0], top + SIZE[1]))
        image.save(OUT / wallpaper.name, quality=85, optimize=True, progressive=True)
    print(OUT / wallpaper.name)
