#!/usr/bin/bash
# List of the image defaults (COSMIC settings and GSettings keys), with a hash
# of each value. celenn-migrate-defaults compares it at boot with the list
# each account last saw, to find the defaults an update changed.
# Runs after every step that changes /usr/share/cosmic.
set -euox pipefail

/usr/libexec/celenn-migrate-defaults --manifest >/usr/share/celenn/defaults.list
wc -l /usr/share/celenn/defaults.list
grep ' com.system76.CosmicTk/v1/icon_theme$' /usr/share/celenn/defaults.list
