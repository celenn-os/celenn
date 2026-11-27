#!/usr/bin/bash
# Checks the account part of celenn-migrate-names on a made-up account that
# still has the settings of cosmic-blue. Run by 99-tests.sh; also runs outside
# the image:
#   build_files/migrate-names-test.sh system_files/usr/libexec/celenn-migrate-names
set -euo pipefail

SCRIPT="$(realpath "${1:-/usr/libexec/celenn-migrate-names}")"
WORK="$(mktemp -d)"
trap 'rm -rf "${WORK}"' EXIT
export HOME="${WORK}/home"
unset XDG_CONFIG_HOME XDG_STATE_HOME
# GSettings keys are not part of this test
mkdir -p "${WORK}/bin"
printf '#!/bin/sh\nexit 1\n' >"${WORK}/bin/dconf"
chmod +x "${WORK}/bin/dconf"
export PATH="${WORK}/bin:${PATH}"
CONFIG="${HOME}/.config"
COSMIC="${CONFIG}/cosmic"

fail() { echo "migrate-names-test: $*" >&2; exit 1; }
put() { # file value
    mkdir -p "$(dirname "$1")"
    printf '%s' "$2" >"$1"
}
is() { # file value
    [[ "$(cat "$1")" == "$2" ]] || fail "$1 is '$(cat "$1")', not '$2'"
}

put "${CONFIG}/cosmic-blue/tools.json" '{"seen": true}'
put "${HOME}/.local/state/cosmic-blue/defaults.list" 'manifest x'
put "${COSMIC}/com.system76.CosmicAppList/v1/favorites" \
    '["firefox", "io.github.cosmic_blue_os.Tools", "com.system76.CosmicFiles"]'
put "${COSMIC}/com.system76.CosmicBackground/v1/all" \
    '(output: "all", source: Path("/usr/share/backgrounds/cosmic-blue/cosmic-blue-arcs.jpg"))'
put "${COSMIC}/com.system76.CosmicTk/v1/icon_theme" '"cosmic-blue"'
put "${COSMIC}/com.system76.CosmicTheme.Dark/v1/icon_theme" '"cosmic-blue"'
put "${COSMIC}/com.system76.CosmicTerm/v1/syntax_theme_dark" '"cosmic-blue"'
put "${COSMIC}/com.system76.CosmicAppLibrary/v1/groups" '[(name: "cosmic-blue", icon: "cosmic-blue-logo-symbolic")]'
put "${COSMIC}/com.system76.CosmicPanel.Dock/v1/opacity" '0.8'
# Already on the new names: both the old and the new folder
put "${CONFIG}/celenn/kept" 'new'

"${SCRIPT}" --user >/dev/null

[[ ! -e "${CONFIG}/cosmic-blue" ]] || fail "the old settings folder is still there"
is "${CONFIG}/celenn/tools.json" '{"seen": true}'
is "${CONFIG}/celenn/kept" 'new'
is "${HOME}/.local/state/celenn/defaults.list" 'manifest x'
is "${COSMIC}/com.system76.CosmicAppList/v1/favorites" \
    '["firefox", "io.github.celenn_os.Tools", "com.system76.CosmicFiles"]'
is "${COSMIC}/com.system76.CosmicBackground/v1/all" \
    '(output: "all", source: Path("/usr/share/backgrounds/celenn/celenn-arcs.jpg"))'
is "${COSMIC}/com.system76.CosmicTk/v1/icon_theme" '"celenn"'
is "${COSMIC}/com.system76.CosmicTheme.Dark/v1/icon_theme" '"celenn"'
is "${COSMIC}/com.system76.CosmicTerm/v1/syntax_theme_dark" '"Celenn"'
is "${COSMIC}/com.system76.CosmicAppLibrary/v1/groups" '[(name: "Celenn OS", icon: "celenn-logo-symbolic")]'
is "${COSMIC}/com.system76.CosmicPanel.Dock/v1/opacity" '0.8'

# A second run changes nothing
before="$(find "${HOME}" -type f -exec sha256sum {} + | sort)"
"${SCRIPT}" --user >/dev/null
[[ "$(find "${HOME}" -type f -exec sha256sum {} + | sort)" == "${before}" ]] || fail "a second run changed files"

echo "migrate-names-test: OK"
