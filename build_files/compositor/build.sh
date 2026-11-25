#!/usr/bin/bash
# Rebuilds Fedora's cosmic-comp package (the COSMIC compositor, also used by
# the login screen) with a black background color, from the same source
# package and build flags as Fedora. Runs in a Fedora container of the same
# release as the image; the binary is written to /out and installed by
# build_files/23-compositor.sh.
#
# Why: until the wallpaper is drawn, cosmic-comp fills the screen with a
# fixed dark gray (CLEAR_COLOR in src/backend/render/mod.rs, "TODO use a
# theme neutral color" upstream). At startup and at each login that gray
# showed for about a second between black screens (boot screen, session
# switch) and the blue wallpaper. Black continues the boot screen instead.
# The replacement must match exactly once: when a new cosmic-comp changes
# that line, this build fails and the COSMIC update pull request waits for
# the "before" text to be updated.
set -euox pipefail

TOPDIR=/rpmbuild
SPEC="${TOPDIR}/SPECS/cosmic-comp.spec"

dnf5 -y install rpm-build dnf5-plugins python3
mkdir -p /srpm "${TOPDIR}"
dnf5 -y download --srpm --destdir /srpm \
    --enablerepo=fedora-source --enablerepo=updates-source \
    cosmic-comp
dnf5 -y builddep /srpm/cosmic-comp-*.src.rpm
rpm -i --define "_topdir ${TOPDIR}" /srpm/cosmic-comp-*.src.rpm

# Unpack the sources (vendored crates included) and apply Fedora's patches
rpmbuild --define "_topdir ${TOPDIR}" -bp "${SPEC}"

src=""
while IFS= read -r manifest; do
    if grep -q '^name = "cosmic-comp"' "${manifest}"; then
        src="$(dirname "${manifest}")"
        break
    fi
done < <(find "${TOPDIR}/BUILD" -name Cargo.toml -not -path '*/vendor/*')
test -n "${src}"

python3 - "${src}/src/backend/render/mod.rs" <<'PY'
import sys
path = sys.argv[1]
before = "pub static CLEAR_COLOR: Color32F = Color32F::new(0.153, 0.161, 0.165, 1.0);"
after = ("// cosmic-blue: black, as the boot screen, until the wallpaper is drawn\n"
         "pub static CLEAR_COLOR: Color32F = Color32F::new(0.0, 0.0, 0.0, 1.0);")
text = open(path, encoding="utf-8").read()
if text.count(before) != 1:
    sys.exit(f"{path}: the CLEAR_COLOR line to replace was found {text.count(before)} times, not once")
open(path, "w", encoding="utf-8").write(text.replace(before, after))
PY

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
# Build only (no new %prep, which would undo the change)
rpmbuild --define "_topdir ${TOPDIR}" -bc --short-circuit "${SPEC}"

bin="$(find "${src}/target/" -type f -newer /tmp/build-start -name cosmic-comp -perm -u+x | head -n1)"
test -n "${bin}"
mkdir -p /out
# Without debug data: Fedora's Rust build flags keep it (-Cdebuginfo=2), and
# only its %install step, which is not run here, strips it
install -m 0755 -s "${bin}" /out/cosmic-comp
# The version the binary was built from, checked against the image's package
rpm -qp --qf '%{VERSION}-%{RELEASE}\n' /srpm/cosmic-comp-*.src.rpm > /out/version
cat /out/version
