#!/usr/bin/bash
# Disable all third-party repos and clean up.
set -euox pipefail

shopt -s nullglob
for repo_file in \
    /etc/yum.repos.d/tailscale.repo \
    /etc/yum.repos.d/_copr*.repo \
    /etc/yum.repos.d/rpmfusion-*.repo \
    /etc/yum.repos.d/negativo17-*.repo \
    /etc/yum.repos.d/fedora-cisco-openh264.repo \
    /usr/share/dnf5/repos.d/*.repo; do
    [[ -f "${repo_file}" ]] || continue
    case "$(basename "${repo_file}")" in
        fedora-cisco-openh264.repo) ;;
        fedora*.repo) continue ;;  # official Fedora repos: leave them as they are
    esac
    sed -i -E 's@^enabled[[:space:]]*=[[:space:]]*(1|true)@enabled=0@' "${repo_file}"
done

# Fedora 45+ ships its repo files in /usr/share/dnf5/repos.d only. bootc-image-builder
# reads /etc/yum.repos.d to build the installer ISO, so expose the Fedora repos there
# too (a file in /etc/yum.repos.d overrides the same-named one in /usr/share).
for repo_file in /usr/share/dnf5/repos.d/fedora*.repo; do
    [[ -f "${repo_file}" ]] || continue
    [[ -f "/etc/yum.repos.d/$(basename "${repo_file}")" ]] || cp -a "${repo_file}" /etc/yum.repos.d/
done

# Any other repository still on (e.g. base-main's ublue-os/akmods COPR, whose
# file name the globs above may miss): off too. 99-tests.sh checks the result.
while read -r repo; do
    case "${repo}" in
        fedora | updates | updates-archive | fedora-multimedia) ;;
        *) dnf5 config-manager setopt "${repo}.enabled=0" ;;
    esac
done < <(dnf5 repolist --enabled | awk 'NR > 1 { print $1 }')

dnf5 clean all
rm -rf /var/lib/dnf /var/cache/* /tmp/* || true
