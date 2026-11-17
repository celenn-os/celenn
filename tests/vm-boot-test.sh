#!/usr/bin/bash
# Boot smoke test: installs the image to a virtual disk with `bootc install`,
# boots it in QEMU (UEFI with Secure Boot, KVM) and checks over SSH that the
# system starts cleanly and that the COSMIC login screen is up. Then logs in a test user
# and takes screenshots of the desktop (desktop_tour in vm-lib.sh), checks that
# no COSMIC program crashed and runs the user's ujust recipes.
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
# Startup time and memory in use at the login screen, to follow over time
# (a virtual machine: compare runs with each other, not with real computers)
boot_time="$(vm "systemd-analyze time" 2>/dev/null | head -n 1 || true)"
memory="$(vm "free -m" 2>/dev/null | awk '/^Mem/ { print $3 " MB used, " $7 " MB available" }' || true)"
greeter="$(vm "systemctl show cosmic-greeter.service -P ActiveEnterTimestampMonotonic" 2>/dev/null || true)"
[[ "${greeter}" =~ ^[0-9]+$ ]] && greeter="$(awk -v us="${greeter}" 'BEGIN { printf "%.1f s", us / 1e6 }')" || greeter=""
log "Boot: ${boot_time:-unknown}; login screen started at ${greeter:-unknown} (from power on of the kernel); memory at the login screen: ${memory:-unknown}"
# What the startup waited for, slowest first
vm "systemd-analyze critical-chain; systemd-analyze blame | head -n 15" || true
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

# Network exposure: the firewall is on, and only known services listen on
# the network. Desktop firewall zones (such as Fedora Workstation's) let
# other computers reach any port above 1024, so a new service listening there
# would be reachable on every network the computer joins. sshd only runs for
# these tests.
firewall="$(vm 'firewall-cmd --state && firewall-cmd --list-all' 2>&1)" \
    || { echo "::error::firewalld is not running: ${firewall}"; errors=1; }
echo "Firewall (default zone): ${firewall}"
listeners="$(vm "ss -H -tulnp" 2>&1)" || listeners=""
echo "${listeners}"
unexpected="$(awk '$5 !~ /^(127\.|\[::1\]|\[::ffff:127\.)/' <<<"${listeners}" \
    | grep -vE 'users:\(\("(sshd|sshd-session|avahi-daemon|NetworkManager|chronyd|tailscaled|systemd-resolve|cupsd|cups-browsed)"' || true)"
if [[ -n "${unexpected}" ]]; then
    echo "::error::Unexpected services listening on the network:"
    echo "${unexpected}"
    errors=1
fi

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
    desktop_tour || { echo "::error::The test user could not log in to a COSMIC session"; errors=1; }
    vm_check_crashes boot || errors=1
    if vm "id ${DEMO_USER}" >/dev/null 2>&1; then
        # The user's own runtime folder (just writes there), not root's from SSH
        ujust_checks="$(vm "runuser -u ${DEMO_USER} -- env -u XDG_RUNTIME_DIR ${DEMO_ENV:-} bash -s" \
            < "${TESTS_DIR}/vm-ujust-checks.sh" 2>&1)" \
            || ujust_checks="FAIL could not run the checks as ${DEMO_USER}: ${ujust_checks}"
        echo "${ujust_checks}"
        while IFS= read -r line; do
            [[ "${line}" == "FAIL "* ]] && { echo "::error::${line#FAIL }"; errors=1; }
        done <<< "${ujust_checks}"
    fi
    demo_journal
fi

(( errors == 0 )) || fail "Boot test failed (see the logs and screenshot in the artifacts)"
log "Boot test passed"
