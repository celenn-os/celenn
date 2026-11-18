#!/usr/bin/bash
# Packages added by Bluefin (build_files/base/04-packages.sh), minus GNOME-specific ones.
set -euox pipefail

FEDORA_PACKAGES=(
    adcli
    adwaita-fonts-all
    autofs
    bash-color-prompt
    bcache-tools
    borgbackup
    containerd
    cryfs
    davfs2
    ddcutil
    evtest
    fastfetch
    firewall-config
    fish
    foo2zjs
    fuse-encfs
    fuse-libs  # libfuse.so.2: runs AppImages (Gear Lever)
    gcc
    gcc-c++
    git-credential-libsecret
    glow
    gnupg2-scdaemon
    gum
    hplip
    ibus-mozc
    ifuse
    igt-gpu-tools
    input-remapper
    iwd
    jetbrains-mono-fonts-all
    just
    krb5-workstation
    libappindicator-gtk3
    libayatana-appindicator-gtk3
    libimobiledevice
    libratbag-ratbagd
    libxcrypt-compat
    lm_sensors
    make
    mesa-libGLU
    mokutil  # ujust enroll-secure-boot-key (Secure Boot keys)
    mozc
    oddjob-mkhomedir
    opendyslexic-fonts
    openssh-askpass
    openssl
    powerstat
    powertop
    printer-driver-brlaser
    pulseaudio-utils
    python3-pip
    python3-pygit2
    rclone
    restic
    rsync
    samba
    samba-dcerpc
    samba-ldb-ldap-modules
    samba-winbind-clients
    samba-winbind-modules
    setools-console
    sssd-nfs-idmap
    switcheroo-control
    tmux
    usbip
    usbmuxd
    waypipe
    wireguard-tools
    wl-clipboard
    xdg-terminal-exec
    xprop
    zenity
    zsh
)
dnf5 -y install "${FEDORA_PACKAGES[@]}"

# Everyday pieces that Fedora Workstation (and so Bluefin) has but our
# base-main + COSMIC image lacks. Codecs, CUPS, SANE, Liberation and the
# default Fedora fonts already come with base-main and the COSMIC group.
EVERYDAY_PACKAGES=(
    # Printing: rootless printer setup through polkit (system-config-printer),
    # and the PPD drivers Fedora Workstation ships for older printers
    cups-pk-helper
    foomatic
    foomatic-db-ppds
    # Printer drivers of Fedora Workstation's printing group, for printers
    # without driverless (IPP Everywhere) printing: Epson inkjets, Gutenprint
    # (older Canon, Epson and others), Brother label printers. HP (hplip),
    # Brother lasers (brlaser) and several others are in the list above.
    epson-inkjet-printer-escpr
    epson-inkjet-printer-escpr2
    gutenprint-cups
    ptouch-driver
    # Scanners: the SANE drivers of most USB scanners (sane-airscan already
    # covers network scanners). Document Scanner is a first-boot flatpak.
    sane-backends-drivers-scanners
    # Fonts metric-compatible with Calibri and Cambria, so Office documents
    # keep their layout (Liberation already covers Arial, Times and Courier)
    google-carlito-fonts
    google-crosextra-caladea-fonts
    # Disks: format USB drives, partitions, disk images, SMART data.
    # COSMIC has no disk tool and GNOME Disks is not on Flathub.
    gnome-disk-utility
    # PDF and e-book previews in COSMIC Files (videos already have
    # ffmpegthumbnailer)
    papers-thumbnailer
    gnome-epub-thumbnailer
    # Extracts the Microsoft core fonts for `ujust install-microsoft-fonts`
    cabextract
)
dnf5 -y install "${EVERYDAY_PACKAGES[@]}"

# Tailscale (official Tailscale repo, disabled after use)
dnf5 config-manager addrepo --from-repofile=https://pkgs.tailscale.com/stable/fedora/tailscale.repo
dnf5 config-manager setopt tailscale-stable.enabled=0
dnf5 -y install --enablerepo=tailscale-stable tailscale

# uupd (Universal Blue's update tool) from the ublue-os/packages COPR,
# enabled only during the install, as in Bluefin.
dnf5 -y copr enable ublue-os/packages
dnf5 -y install --repo="copr:copr.fedorainfracloud.org:ublue-os:packages" uupd
dnf5 -y copr disable ublue-os/packages

# Nerd Font symbols (icons of fastfetch and Starship), from the che/nerd-fonts
# COPR as in Bluefin, enabled only during the install
dnf5 -y copr enable che/nerd-fonts
dnf5 -y install --repo="copr:copr.fedorainfracloud.org:che:nerd-fonts" nerd-fonts
dnf5 -y copr disable che/nerd-fonts

# Starship (shell prompt), official binary of a pinned release (updated by
# Renovate), checked against the SHA-256 published with that release
STARSHIP_REPO="https://github.com/starship/starship"
STARSHIP_VERSION="v1.27.0"
STARSHIP_ASSET="starship-x86_64-unknown-linux-gnu.tar.gz"
STARSHIP_URL="${STARSHIP_REPO}/releases/download/${STARSHIP_VERSION}/${STARSHIP_ASSET}"
curl -fsSL --retry 3 -o "/tmp/${STARSHIP_ASSET}" "${STARSHIP_URL}"
curl -fsSL --retry 3 -o "/tmp/${STARSHIP_ASSET}.sha256" "${STARSHIP_URL}.sha256"
echo "$(tr -d '[:space:]' <"/tmp/${STARSHIP_ASSET}.sha256")  /tmp/${STARSHIP_ASSET}" | sha256sum -c -
tar -xzf "/tmp/${STARSHIP_ASSET}" -C /tmp starship
install -m 0755 /tmp/starship /usr/bin/starship
rm -f "/tmp/${STARSHIP_ASSET}" "/tmp/${STARSHIP_ASSET}.sha256" /tmp/starship
starship --version | grep -Fx "starship ${STARSHIP_VERSION#v}"

# Removed packages, as in Bluefin (Firefox comes as a flatpak)
EXCLUDED_PACKAGES=(
    fedora-bookmarks
    fedora-chromium-config
    firefox
    firefox-langpacks
    podman-docker
)
readarray -t INSTALLED < <(rpm -qa --qf '%{NAME}\n' "${EXCLUDED_PACKAGES[@]}")
if [[ "${#INSTALLED[@]}" -gt 0 ]]; then
    dnf5 -y remove "${INSTALLED[@]}"
fi
