#!/usr/bin/bash
# Boot smoke test: installs the image to a virtual disk with `bootc install`,
# boots it in QEMU (UEFI with Secure Boot, KVM) and checks over SSH that the
# system starts cleanly, that the first-boot apps are installed and that the
# COSMIC login screen is up. Then logs in a test user
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
TARGET_IMGREF="${TARGET_IMGREF:-ghcr.io/cosmic-blue-os/cosmic-blue:testing}"
# shellcheck source=tests/vm-lib.sh
source "$(dirname "$0")/vm-lib.sh"
SYSTEM_FILES="$(dirname "$0")/../system_files"

vm_install "${IMAGE}" "${TARGET_IMGREF}"
# A second, empty disk for Other disks (disks_prepare in vm-lib.sh)
truncate -s 1G "${WORK}/data.raw"
# shellcheck disable=SC2054 # QEMU options are comma-separated
VM_EXTRA_ARGS=(-drive file="${WORK}/data.raw",format=raw,if=virtio)
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
# without signature verification: celenn-signed-updates must offer to
# turn it on (dry run: the real switch would download the published image).
# Its timer is stopped first so it does not start that download mid-test.
vm "systemctl stop celenn-signed-updates.timer"
signed_updates="$(vm "/usr/libexec/celenn-signed-updates --dry-run")" || signed_updates=""
echo "${signed_updates}"
grep -q 'would turn on the signature check' <<<"${signed_updates}" \
    || { echo "::error::celenn-signed-updates would not turn on the signature check"; errors=1; }

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

# First-boot apps (flatpak-preinstall.service): every app of the image's list
# (this checkout's, which the image was built from) is installed, and every
# default application (Papers, Loupe, Gear Lever for AppImages...) has its
# launcher
preinstall_problem() { echo "::error::First-boot apps: $*"; errors=1; }
wait_for 600 '[ "$(systemctl show -P ActiveState flatpak-preinstall.service)" != activating ]' \
    || preinstall_problem "flatpak-preinstall.service still runs after 10 minutes"
wanted="$(awk '/^\[Flatpak Preinstall / { app = substr($3, 1, length($3) - 1) }
    /^IsRuntime=false/ { print app }' "${SYSTEM_FILES}/usr/share/flatpak/preinstall.d/celenn.preinstall" | sort -u)"
installed="$(vm 'flatpak list --system --app --columns=application' | sort -u)" || installed=""
echo "Installed flatpaks:"
echo "${installed}"
[[ -n "${wanted}" ]] || preinstall_problem "no app in the list of first-boot apps"
missing="$(comm -23 <(echo "${wanted}") <(echo "${installed}") | xargs)"
[[ -z "${missing}" ]] || preinstall_problem "not installed: ${missing}"
for launcher in $(sed -n '/^\[Default Applications\]/,/^\[/{s/^[a-z][^=]*=//p}' \
        "${SYSTEM_FILES}/etc/xdg/mimeapps.list" | tr ';' '\n' | sort -u); do
    vm "test -e /usr/share/applications/${launcher} || test -e /var/lib/flatpak/exports/share/applications/${launcher}" \
        || preinstall_problem "the default application ${launcher} has no launcher"
done

# `ujust report-bug` lists the image and the booted deployment
bug_report="$(vm "ujust report-bug print")" || bug_report=""
echo "${bug_report}"
grep -qE '^Image: cosmic-blue(-nvidia)? [0-9]' <<<"${bug_report}" \
    && grep -qE '^Booted: [0-9][^ ]* +.*ghcr\.io/cosmic-blue-os/cosmic-blue' <<<"${bug_report}" \
    || { echo "::error::ujust report-bug does not show the image and the booted deployment"; errors=1; }
grep -qE '^Free disk space: [0-9.,]+[KMGT]' <<<"${bug_report}" \
    || { echo "::error::ujust report-bug does not show the free disk space"; errors=1; }

# `ujust switch-channel` reads the channel this system follows (the tag of
# TARGET_IMGREF, as installed above; latest is testing's old name); switching
# itself would download the published image
channel="$(vm "ujust switch-channel status" 2>&1)" || channel=""
echo "${channel}"
expected="${TARGET_IMGREF##*:}"
[[ "${expected}" != latest ]] || expected=testing
case "${expected}" in
    stable|testing|dev)
        grep -q "^Channel: ${expected} " <<<"${channel}" \
            || { echo "::error::ujust switch-channel status does not show the ${expected} channel"; errors=1; } ;;
esac

# Updates app: what it shows (read from rpm-ostree, as in the user's session)
# on a new install: this version, the channel, no previous version yet
updates="$(vm "/usr/libexec/celenn-updates --status" 2>&1)" || updates=""
echo "${updates}"
grep -qE '^current: [0-9]' <<<"${updates}" \
    && grep -qx 'previous: none' <<<"${updates}" \
    || { echo "::error::celenn-updates --status does not show the version"; errors=1; }
case "${expected}" in
    stable|testing|dev)
        grep -qx "channel: ${expected}" <<<"${updates}" \
            || { echo "::error::celenn-updates --status does not show the ${expected} channel"; errors=1; } ;;
esac
# The older stable versions it offers (stable-YYYYMMDD tags of the registry)
older="$(vm "/usr/libexec/celenn-updates --older" 2>&1)" || older=""
echo "Older stable versions: ${older//$'\n'/ }"
grep -qE '^stable-[0-9]{8}$' <<<"${older}" \
    || echo "::warning::celenn-updates --older lists no older stable version (${older})"
# Its root helper only does what it is meant to: no rollback without a
# previous version, no dev channel (ujust only), no tag other than stable-YYYYMMDD
vm "/usr/libexec/celenn-updates-helper rollback" 2>/dev/null \
    && { echo "::error::celenn-updates-helper rolled back without a previous version"; errors=1; }
vm "/usr/libexec/celenn-updates-helper channel dev" 2>/dev/null \
    && { echo "::error::celenn-updates-helper accepted the dev channel"; errors=1; }
vm "/usr/libexec/celenn-updates-helper pin testing" 2>/dev/null \
    && { echo "::error::celenn-updates-helper pinned something else than a stable-YYYYMMDD tag"; errors=1; }
# A version left behind with "Go back" is not installed again by uupd
# (through celenn-bootc), until a newer one is out. The published
# image of TARGET_IMGREF is an update for this test build: set it aside as
# the Updates app's helper would, then pretend another version was.
hold=/var/lib/celenn/update-hold
upgrade_check="$(vm "bootc upgrade --check" 2>&1)" || upgrade_check=""
available="$(sed -n 's/^  Digest: //p' <<<"${upgrade_check}" | head -n 1)"
if [[ -n "${available}" ]]; then
    vm "mkdir -p ${hold%/*} && echo '${available} 44.test' > ${hold}"
    held_check="$(vm "/usr/libexec/celenn-bootc upgrade --check" 2>&1)" || held_check=""
    echo "${held_check}"
    grep -q '^No changes in: ' <<<"${held_check}" && vm "test -s ${hold}" \
        || { echo "::error::celenn-bootc offers the version set aside again"; errors=1; }
    vm "echo 'sha256:0000000000000000000000000000000000000000000000000000000000000000 44.old' > ${hold}"
    new_check="$(vm "/usr/libexec/celenn-bootc upgrade --check" 2>&1)" || new_check=""
    grep -q '^Update available for: ' <<<"${new_check}" && vm "test ! -e ${hold}" \
        || { echo "::error::celenn-bootc does not offer a newer version after one was set aside"; errors=1; }
    vm "rm -f ${hold}"
else
    echo "::warning::No update offered for ${TARGET_IMGREF} (${upgrade_check}): the update hold was not tested"
fi

# FlatPark is off by default. `ujust toggle-flatpark` turns it on at the
# lowest priority with its filter, and off again; the one-time cleanup for
# systems where older images turned it on removes it when no app uses it.
flatpark_state() { # --user (the installation COSMIC Store manages) or --system
    vm "flatpak remotes $1 --columns=name,priority,filter" | awk '$1 == "flatpark" {print $2, $3}'
}
flatpark_problem() { echo "::error::FlatPark: $*"; errors=1; }
vm 'test -e /var/lib/celenn/flatpark-off-done' \
    || flatpark_problem "celenn-flatpark-off.service did not run"
[[ -z "$(flatpark_state --user)$(flatpark_state --system)" ]] || flatpark_problem "on by default"
vm 'ujust toggle-flatpark on' || flatpark_problem "ujust toggle-flatpark on failed"
state="$(flatpark_state --user || true)"
echo "FlatPark priority and filter: ${state}"
[[ "${state}" == "0 /usr/share/flatpak/flatpark.filter" ]] \
    || flatpark_problem "not on at priority 0 with its filter after ujust toggle-flatpark on"
vm 'ujust toggle-flatpark off' || flatpark_problem "ujust toggle-flatpark off failed"
[[ -z "$(flatpark_state --user)" ]] || flatpark_problem "still on after ujust toggle-flatpark off"
# As older images left it: system-wide, from remotes.d
vm 'flatpak remote-add --system flatpark /usr/share/celenn/flatpak/flatpark.flatpakrepo \
        && rm /var/lib/celenn/flatpark-off-done && systemctl start celenn-flatpark-off.service' \
    || flatpark_problem "celenn-flatpark-off.service failed"
[[ -z "$(flatpark_state --system)" ]] || flatpark_problem "celenn-flatpark-off.service left it on"

# Let the login screen draw, then take a picture of it
sleep 20
screenshot login-screen

if (( errors == 0 )); then
    disks_prepare || errors=1
    desktop_tour || { echo "::error::The test user could not log in to a COSMIC session"; errors=1; }
    # A new account: its first login and Firefox must be right (vm-lib.sh)
    for problem in "${DESKTOP_PROBLEMS[@]}"; do
        echo "::error::${problem}"
        errors=1
    done
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
        disks_checks || errors=1
        # Last: it closes the test user's session
        parental_checks || errors=1
    fi
    demo_journal
fi

(( errors == 0 )) || fail "Boot test failed (see the logs and screenshot in the artifacts)"
log "Boot test passed"
