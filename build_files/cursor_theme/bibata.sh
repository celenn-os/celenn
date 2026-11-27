#!/usr/bin/bash
# Build the "Bibata-Modern-celenn" cursor theme into /out: Bibata Modern
# (rounded) from the sources kept in ./bibata (see its README), recolored with
# the Celenn OS colors. Fedora packages Bibata only in its upstream colors.
set -euxo pipefail

# Upstream repository and release the sources in ./bibata come from. Renovate (see
# .github/renovate.json5) opens a pull request on a new release, to be
# completed by hand with the new sources: copy svg/modern (following links),
# configs/normal/x.build.toml and LICENSE from the release tag into ./bibata.
# shellcheck disable=SC2034 # read by Renovate only
BIBATA_REPO="https://github.com/ful1e5/Bibata_Cursor.git"
BIBATA_VERSION="v2.0.7"

dnf5 -y install python3 librsvg2-tools xcursorgen

HERE="$(dirname "$0")"
python3 "${HERE}/bibata.py" "${HERE}/bibata" /out/Bibata-Modern-celenn "${BIBATA_VERSION}"
