#!/usr/bin/bash
# Build uupd (Universal Blue's update tool) into /out from its source at a
# pinned release, with its own Makefile: the binary, its systemd units, its
# polkit rule and its default config, the same files as the ublue-os/packages
# COPR package Bluefin installs. Built here so that the image build does not
# depend on the COPR servers.
set -euxo pipefail

# Release built. Renovate (see .github/renovate.json5) opens a pull request
# on a new release tag.
UUPD_REPO="https://github.com/ublue-os/uupd.git"
UUPD_VERSION="v1.5.9"

dnf5 -y install git golang make

git clone --quiet --depth 1 --branch "${UUPD_VERSION}" "${UUPD_REPO}" /src
cd /src
# Go modules come from the vendor directory of the repository: no download
make build VERSION="${UUPD_VERSION#v}" GO_BUILD_FLAGS="-trimpath -mod=vendor"
make install DESTDIR=/out PREFIX=/usr

test "$(/out/usr/bin/uupd --version 2>&1 | grep -o '[0-9][0-9.]*' | head -n 1)" = "${UUPD_VERSION#v}"
