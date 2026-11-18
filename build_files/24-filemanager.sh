#!/usr/bin/bash
# Replaces Fedora's COSMIC Files binaries with the ones rebuilt from the same
# source package with the cosmic-blue patch (build_files/filemanager): the
# user folders (Documents, Music...) get their own icons under /var/home.
set -euox pipefail

installed="$(rpm -q --qf '%{VERSION}-%{RELEASE}\n' cosmic-files)"
built="$(cat /ctx/filemanager/version)"
if [ "${installed}" != "${built}" ]; then
    echo "cosmic-files ${installed} installed, but ${built} was patched" >&2
    exit 1
fi

for bin in /ctx/filemanager/bin/*; do
    # Only binaries that the package installs
    rpm -qf "/usr/bin/$(basename "${bin}")" >/dev/null
    install -m 0755 "${bin}" "/usr/bin/$(basename "${bin}")"
done
