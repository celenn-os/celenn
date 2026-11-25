#!/usr/bin/bash
# Replaces Fedora's cosmic-comp binary (the COSMIC compositor, also used by the
# login screen) with the one rebuilt from the same source package with a black
# background until the wallpaper is drawn (see build_files/compositor/build.sh).
set -euox pipefail

installed="$(rpm -q --qf '%{VERSION}-%{RELEASE}\n' cosmic-comp)"
built="$(cat /ctx/compositor/version)"
if [ "${installed}" != "${built}" ]; then
    echo "cosmic-comp ${installed} installed, but ${built} was rebuilt" >&2
    exit 1
fi

install -m 0755 /ctx/compositor/cosmic-comp /usr/bin/cosmic-comp
