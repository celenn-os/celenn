#!/usr/bin/bash
# Build the "Bibata-Modern-cosmic-blue" cursor theme into /out: Bibata Modern
# (rounded) from the sources kept in ./bibata (see its README), recolored with
# the cosmic-blue colors. Fedora packages Bibata only in its upstream colors.
set -euxo pipefail

dnf5 -y install librsvg2-tools xcursorgen

HERE="$(dirname "$0")"
python3 "${HERE}/bibata.py" "${HERE}/bibata" /out/Bibata-Modern-cosmic-blue v2.0.7
