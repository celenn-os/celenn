#!/usr/bin/bash
# Rebuilds Fedora's cosmic-files package with the cosmic-blue patch (see
# patch.py), from the same source package and build flags as Fedora. Runs in a
# Fedora container of the same release as the image; the binary is written to
# /out and installed by build_files/24-filemanager.sh.
set -euox pipefail

TOPDIR=/rpmbuild
SPEC="${TOPDIR}/SPECS/cosmic-files.spec"

dnf5 -y install rpm-build dnf5-plugins python3
mkdir -p /srpm "${TOPDIR}"
dnf5 -y download --srpm --destdir /srpm \
    --enablerepo=fedora-source --enablerepo=updates-source \
    cosmic-files
dnf5 -y builddep /srpm/cosmic-files-*.src.rpm
rpm -i --define "_topdir ${TOPDIR}" /srpm/cosmic-files-*.src.rpm

# Unpack the sources (vendored crates included) and apply Fedora's patches
rpmbuild --define "_topdir ${TOPDIR}" -bp "${SPEC}"

src=""
while IFS= read -r manifest; do
    if grep -q '^name = "cosmic-files"' "${manifest}"; then
        src="$(dirname "${manifest}")"
        break
    fi
done < <(find "${TOPDIR}/BUILD" -name Cargo.toml -not -path '*/vendor/*')
test -n "${src}"

python3 /filemanager/patch.py "${src}"

# Build only (no new %prep, which would undo the patch)
rpmbuild --define "_topdir ${TOPDIR}" -bc --short-circuit "${SPEC}"

# The file manager, and the applet (desktop icons) that shares its code
mkdir -p /out/bin
for name in cosmic-files cosmic-files-applet; do
    bin="$(find "${src}/target" -type f -name "${name}" -perm -u+x | head -n1)"
    if [ -n "${bin}" ]; then
        install -m 0755 "${bin}" "/out/bin/${name}"
    fi
done
test -x /out/bin/cosmic-files
# The version the binary was built from, checked against the image's package
rpm -qp --qf '%{VERSION}-%{RELEASE}\n' /srpm/cosmic-files-*.src.rpm > /out/version
cat /out/version
