#!/usr/bin/bash
# Replaces Fedora's cosmic-store binary with the one rebuilt from the same
# source package with the Celenn OS patch (build_files/store): FlatPark is
# listed as a recommended Flatpak source, off until the user adds it.
set -euox pipefail

installed="$(rpm -q --qf '%{VERSION}-%{RELEASE}\n' cosmic-store)"
built="$(cat /ctx/store/version)"
if [ "${installed}" != "${built}" ]; then
    echo "cosmic-store ${installed} installed, but ${built} was patched" >&2
    exit 1
fi

install -m 0755 /ctx/store/cosmic-store /usr/bin/cosmic-store
