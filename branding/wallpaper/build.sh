#!/usr/bin/bash
# Rebuild the cosmic-blue wallpapers from their sources.
# Needs python3 with numpy, pillow and scipy, node with playwright, and curl.
set -euo pipefail
cd "$(dirname "$0")"
out=../../system_files/usr/share/backgrounds/cosmic-blue

# The logo scene
python3 sky.py
python3 foreground.py
node render.js
python3 compose.py dark "${out}/cosmic-blue.jpg"

# A planetary system, over the same sky (the default until 2026-11)
python3 orbits.py "${out}/cosmic-blue-orbits.jpg"

# Waves of light
python3 concept.py waves concept.svg
node render.js concept.svg concept.png
python3 compose.py concept "${out}/cosmic-blue-waves.jpg" concept.png

# The Milky Way above the Earth (NASA photo)
curl -fsSL -o milky-way-orig.jpg \
    https://images-assets.nasa.gov/image/iss064e055946/iss064e055946~orig.jpg
python3 milky_way.py milky-way-orig.jpg "${out}/cosmic-blue-milky-way.jpg"

# Shader wallpapers (GLSL, rendered with headless Chromium): a spiral, a galaxy,
# a river of stars, a nebula and orbit lines
# (shader name:file name; the file names set the order in COSMIC Settings)
for pair in spiral:deep-spiral galaxy:galaxy star-river:galactic-river \
        nebula:gas-veils orbit-lines:line-orbits; do
    name="${pair%%:*}"
    node shaders/render.js "${name}" "${name}.png"
    python3 -c 'import sys; from PIL import Image; Image.open(sys.argv[1]).convert("RGB").save(sys.argv[2], quality=90, optimize=True)' \
        "${name}.png" "${out}/cosmic-blue-${pair#*:}.jpg"
    rm -f "${name}.png"
done
# The default: the logo mark, its orbit echoing outwards, the name in a corner
SHADER_DEFINES="$(python3 logo_mark.py --defines)" node shaders/render.js echoes echoes.png
python3 logo_mark.py echoes.png "${out}/cosmic-blue-arcs.jpg" corner
rm -f echoes.png

rm -f sky.png foreground.svg foreground.png concept.svg concept.png milky-way-orig.jpg
