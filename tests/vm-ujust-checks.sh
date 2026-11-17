#!/usr/bin/bash
# cosmic-blue's ujust recipes that change the user's COSMIC settings, run as
# the test user by tests/vm-boot-test.sh (after the desktop tour, so the
# account has a real COSMIC configuration). Prints one "FAIL <reason>" line
# per problem.
set -uo pipefail
cd ~ || { echo "FAIL no home folder"; exit 0; }

config="${XDG_CONFIG_HOME:-${HOME}/.config}/cosmic"
mode="${config}/com.system76.CosmicTheme.Mode/v1/is_dark"
keyboard="${config}/com.system76.CosmicComp/v1/xkb_config"
# A setting of no COSMIC component, to see what each reset keeps
other="${config}/cosmic-blue-ujust-test/v1/kept"

fail() { echo "FAIL $*"; }
setting() { cat "$1" 2>/dev/null || echo "(none)"; }
run() { # recipe and arguments
    local output
    output="$(ujust "$@" 2>&1)" || fail "ujust $* failed: ${output}"
    echo "ujust $*: ${output}"
}

# toggle-dark-mode: light, dark again, then a forced mode
run toggle-dark-mode
[[ "$(setting "${mode}")" == false ]] \
    || fail "ujust toggle-dark-mode did not turn the light theme on (is_dark: $(setting "${mode}"))"
run toggle-dark-mode
[[ "$(setting "${mode}")" == true ]] \
    || fail "a second ujust toggle-dark-mode did not turn the dark theme back on (is_dark: $(setting "${mode}"))"
run toggle-dark-mode light
[[ "$(setting "${mode}")" == false ]] \
    || fail "ujust toggle-dark-mode light did not turn the light theme on (is_dark: $(setting "${mode}"))"

# reset-cosmic-theme: the appearance goes back to the image defaults, other
# settings stay
install -D -m 0644 /dev/null "${other}"
run reset-cosmic-theme yes
[[ ! -e "${config}/com.system76.CosmicTheme.Mode" ]] \
    || fail "ujust reset-cosmic-theme left the dark/light choice (is_dark: $(setting "${mode}"))"
[[ -e "${other}" ]] || fail "ujust reset-cosmic-theme removed other settings"

# reset-cosmic-config: every setting moved to a backup, except the keyboard
# layout (and clock format) set from the installer
keyboard_before="$(setting "${keyboard}")"
run reset-cosmic-config yes
backup="$(compgen -G "${config}.backup-*" | tail -n 1)"
if [[ -z "${backup}" ]]; then
    fail "ujust reset-cosmic-config made no backup"
else
    [[ -e "${backup}/cosmic-blue-ujust-test/v1/kept" ]] \
        || fail "ujust reset-cosmic-config backup ${backup} lacks the settings"
fi
[[ ! -e "${other}" ]] || fail "ujust reset-cosmic-config kept the other settings"
[[ "$(setting "${keyboard}")" == "${keyboard_before}" ]] \
    || fail "ujust reset-cosmic-config did not keep the keyboard layout"
rm -rf "${backup:-/nonexistent}"
