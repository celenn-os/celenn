<p align="center"><img src="branding/logo/cosmic-blue-readme-banner.svg" alt="cosmic-blue" width="720"></p>

cosmic-blue is an independent Fedora Atomic image with the
[COSMIC](https://system76.com/cosmic) desktop. It is built on the
[Universal Blue](https://universal-blue.org/) base images and inspired by
Bluefin, but it is not part of, affiliated with or endorsed by Universal Blue
or Bluefin.

It is an atomic (bootc) system: updates are applied in the background, take
effect on the next reboot, and can be rolled back.

[![cosmic-blue desktop with a terminal open](https://cosmic-blue-os.github.io/cosmic-blue/img/screenshots/terminal.webp)](https://cosmic-blue-os.github.io/cosmic-blue/screenshots/)

## Versions

<!-- versions:start (written by release-notes.yml on each stable release) -->
In the current `stable` image (cosmic-blue 44.20261119.6):

![Fedora 44](https://img.shields.io/badge/Fedora-44-4f8dff)
![Kernel 7.3.4](https://img.shields.io/badge/Kernel-7.3.4-4f8dff)
![COSMIC 1.11.0](https://img.shields.io/badge/COSMIC-1.11.0-4f8dff)
![Mesa 26.2.7](https://img.shields.io/badge/Mesa-26.2.7-4f8dff)
![NVIDIA 620.70.06](https://img.shields.io/badge/NVIDIA-620.70.06-4f8dff)
<!-- versions:end -->

What changed in each `stable` version (packages, COSMIC, cosmic-blue):
[Releases](https://github.com/cosmic-blue-os/cosmic-blue/releases).

## Install

**New install:** download the live ISO from the
[website](https://cosmic-blue-os.github.io/cosmic-blue/), write it to a USB
stick with [Fedora Media Writer](https://fedoraproject.org/workstation/download),
boot it and start the installer from the desktop icon.

**From another bootc system** (Bluefin, Aurora, Bazzite, Fedora Atomic...):

```bash
sudo bootc switch ghcr.io/cosmic-blue-os/cosmic-blue:stable
sudo systemctl reboot
```

After the reboot, cosmic-blue turns on the signature check of its updates by
itself (`bootc switch --enforce-container-sigpolicy`).

`stable` moves once a week to a build that has spent a week on `testing`.
`testing` gets every daily build, and `dev` every merged change: switch with
`ujust switch-channel testing` (or `dev`) to get changes sooner.

**NVIDIA graphics cards** (GTX 16xx, RTX 20xx and newer): use the
`cosmic-blue-nvidia` image, which ships NVIDIA's driver. For a new install,
choose NVIDIA on the website. From an installed system:

```bash
ujust enroll-secure-boot-key   # only if Secure Boot is on
sudo bootc switch ghcr.io/cosmic-blue-os/cosmic-blue-nvidia:stable
sudo systemctl reboot
```

Secure Boot: the standard image works with it as is (Fedora's signed kernel).
Only the NVIDIA image needs its driver's key enrolled once: on the first reboot,
a blue screen (MOK manager) shows up; choose *Enroll MOK*, then *Continue*, and
type the password `cosmicblue` (QWERTY keyboard layout).

## Updates and rollback

```bash
ujust update          # update now instead of waiting for the automatic update
sudo bootc rollback   # go back to the previous version on next boot
```

## Developer tools

```bash
ujust install-dev-tools   # Docker, VS Code, Podman Desktop and other developer tools
```

## Image signature

Images are signed with [cosign](https://github.com/sigstore/cosign) and
installed systems refuse unsigned updates. To check by hand with
[`cosign.pub`](cosign.pub):

```bash
cosign verify --key cosign.pub ghcr.io/cosmic-blue-os/cosmic-blue:stable
```

## Credits

Built on [Universal Blue](https://github.com/ublue-os),
[Bluefin](https://github.com/projectbluefin), [Fedora](https://fedoraproject.org/)
and [COSMIC](https://github.com/pop-os/cosmic-epoch), from the
[ublue-os/image-template](https://github.com/ublue-os/image-template) repository.
The Milky Way wallpaper is a photo taken on the International Space Station,
courtesy of NASA.

cosmic-blue is an independent project, not affiliated with any of the projects
above.
