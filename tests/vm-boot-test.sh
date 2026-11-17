#!/usr/bin/bash
# Boot smoke test: installs the image to a virtual disk with `bootc install`,
# boots it in QEMU (UEFI with Secure Boot, KVM) and checks over SSH that the
# system starts cleanly and that the COSMIC login screen is up. Then logs in a test user
# and takes screenshots of the desktop (desktop_tour in vm-lib.sh).
#
# Usage: tests/vm-boot-test.sh <image> <output dir>
#   <image> must be in the root (sudo) podman storage.
# Needs: podman, qemu-system-x86_64, OVMF, socat, openssh-client, jq, KVM.
set -euo pipefail
trap 'echo "::error::Command failed at line ${LINENO}: ${BASH_COMMAND}"' ERR

IMAGE="$1"
OUT="$(realpath -m "$2")"
TARGET_IMGREF="${TARGET_IMGREF:-ghcr.io/cosmic-blue-os/cosmic-blue:latest}"
# shellcheck source=tests/vm-lib.sh
source "$(dirname "$0")/vm-lib.sh"

vm_install "${IMAGE}" "${TARGET_IMGREF}"
vm_start
vm_wait_ssh

errors=0
vm_check_system boot || errors=1
vm_check_tuning boot || errors=1
vm "bootc status --json" | jq -e --arg ref "${TARGET_IMGREF}" \
    '.status.booted.image.image.image == $ref' >/dev/null \
    || { echo "::error::The booted image is not ${TARGET_IMGREF}"; errors=1; }
# `bootc install` here, like a manual `bootc switch`, records the image
# without signature verification: cosmic-blue-signed-updates must offer to
# turn it on (dry run: the real switch would download the published image).
# Its timer is stopped first so it does not start that download mid-test.
vm "systemctl stop cosmic-blue-signed-updates.timer"
signed_updates="$(vm "/usr/libexec/cosmic-blue-signed-updates --dry-run")" || signed_updates=""
echo "${signed_updates}"
grep -q 'would turn on the signature check' <<<"${signed_updates}" \
    || { echo "::error::cosmic-blue-signed-updates would not turn on the signature check"; errors=1; }

# `ujust bug-report` lists the image and the booted deployment
bug_report="$(vm "ujust bug-report print")" || bug_report=""
echo "${bug_report}"
grep -qE '^Image: cosmic-blue(-nvidia)? [0-9]' <<<"${bug_report}" \
    && grep -qE '^Booted: [0-9.]+ +.*ghcr\.io/cosmic-blue-os/cosmic-blue' <<<"${bug_report}" \
    || { echo "::error::ujust bug-report does not show the image and the booted deployment"; errors=1; }

# FlatPark is off by default. `ujust toggle-flatpark` turns it on at the
# lowest priority with its filter, and off again; the one-time cleanup for
# systems where older images turned it on removes it when no app uses it.
flatpark_state() { # --user (the installation COSMIC Store manages) or --system
    vm "flatpak remotes $1 --columns=name,priority,filter" | awk '$1 == "flatpark" {print $2, $3}'
}
flatpark_problem() { echo "::error::FlatPark: $*"; errors=1; }
vm 'test -e /var/lib/cosmic-blue/flatpark-off-done' \
    || flatpark_problem "cosmic-blue-flatpark-off.service did not run"
[[ -z "$(flatpark_state --user)$(flatpark_state --system)" ]] || flatpark_problem "on by default"
vm 'ujust toggle-flatpark on' || flatpark_problem "ujust toggle-flatpark on failed"
state="$(flatpark_state --user || true)"
echo "FlatPark priority and filter: ${state}"
[[ "${state}" == "0 /usr/share/flatpak/flatpark.filter" ]] \
    || flatpark_problem "not on at priority 0 with its filter after ujust toggle-flatpark on"
vm 'ujust toggle-flatpark off' || flatpark_problem "ujust toggle-flatpark off failed"
[[ -z "$(flatpark_state --user)" ]] || flatpark_problem "still on after ujust toggle-flatpark off"
# As older images left it: system-wide, from remotes.d
vm 'flatpak remote-add --system flatpark /usr/share/cosmic-blue/flatpak/flatpark.flatpakrepo \
        && rm /var/lib/cosmic-blue/flatpark-off-done && systemctl start cosmic-blue-flatpark-off.service' \
    || flatpark_problem "cosmic-blue-flatpark-off.service failed"
[[ -z "$(flatpark_state --system)" ]] || flatpark_problem "cosmic-blue-flatpark-off.service left it on"

# Let the login screen draw, then take a picture of it
sleep 20
screenshot login-screen

if (( errors == 0 )); then
    desktop_tour || true
    demo_journal
fi

(( errors == 0 )) || fail "Boot test failed (see the logs and screenshot in the artifacts)"
log "Boot test passed"
