#!/usr/bin/bash
# Rebuilds Fedora's cosmic-store package with the cosmic-blue patch (see
# patch.py), from the same source package and build flags as Fedora. Runs in a
# Fedora container of the same release as the image; the binary is written to
# /out and installed by build_files/24-store.sh.
set -euox pipefail

TOPDIR=/rpmbuild
SPEC="${TOPDIR}/SPECS/cosmic-store.spec"

dnf5 -y install rpm-build dnf5-plugins python3
mkdir -p /srpm "${TOPDIR}"
dnf5 -y download --srpm --destdir /srpm \
    --enablerepo=fedora-source --enablerepo=updates-source \
    cosmic-store
dnf5 -y builddep /srpm/cosmic-store-*.src.rpm
rpm -i --define "_topdir ${TOPDIR}" /srpm/cosmic-store-*.src.rpm

# Unpack the sources (vendored crates included) and apply Fedora's patches
rpmbuild --define "_topdir ${TOPDIR}" -bp "${SPEC}"

src=""
while IFS= read -r manifest; do
    if grep -q '^name = "cosmic-store"' "${manifest}"; then
        src="$(dirname "${manifest}")"
        break
    fi
done < <(find "${TOPDIR}/BUILD" -name Cargo.toml -not -path '*/vendor/*')
test -n "${src}"

python3 /store/patch.py "${src}"

# Compiled crates kept from the previous build in CARGO_CACHE_DIR
# (Containerfile.helpers). The package's own sources are touched, so they
# always compile again even when the unpacked files carry older dates.
if [ -n "${CARGO_CACHE_DIR:-}" ]; then
    mkdir -p "${CARGO_CACHE_DIR}/target"
    rm -rf "${src}/target"
    ln -s "${CARGO_CACHE_DIR}/target" "${src}/target"
    find "${src}" \( -path '*/vendor' -o -path "${src}/target" \) -prune -o -type f -exec touch {} +
fi

# Binaries older than this build (kept in CARGO_CACHE_DIR) are not used
touch /tmp/build-start
# Build only (no new %prep, which would undo the patch)
rpmbuild --define "_topdir ${TOPDIR}" -bc --short-circuit "${SPEC}"

bin="$(find "${src}/target/" -type f -newer /tmp/build-start -name cosmic-store -perm -u+x | head -n1)"
test -n "${bin}"
# The patch is in the binary (FlatPark, translations)
grep -q /usr/share/cosmic-blue/flatpak/flatpark.flatpakrepo "${bin}"
for text in "Mise à jour du système" "Auf Version" "Actualización del sistema" \
    "Aggiornamento del sistema" "Atualização do sistema"; do
    grep -q "${text}" "${bin}"
done
mkdir -p /out
install -m 0755 "${bin}" /out/cosmic-store
# The version the binary was built from, checked against the image's package
rpm -qp --qf '%{VERSION}-%{RELEASE}\n' /srpm/cosmic-store-*.src.rpm > /out/version
cat /out/version
