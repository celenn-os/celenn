# cosmic-blue: a Bluefin-like Universal Blue image with COSMIC instead of GNOME.
# See docs/plan.md for the choice of base image and the inventory.

# Fedora release: the one of Bluefin's stable tag. check-cosmic-version.yml
# opens the pull request that moves BASE_TAG and BASE_IMAGE together when
# Bluefin stable moves to a new release (in 2026-11, Fedora 45 still broke the
# live ISO installer: systemd-tmpfiles exited with code 73).
# Images are followed by tag, not pinned by digest: every daily build takes
# the latest base, common and brew images (see .github/renovate.json5).
# BASE_TAG is the Fedora release of Containerfile.helpers (`just fedora-version`).
ARG BASE_TAG="44"
ARG BASE_IMAGE="ghcr.io/ublue-os/base-main:44"
ARG COMMON_IMAGE="ghcr.io/projectbluefin/common:latest"
ARG BREW_IMAGE="ghcr.io/ublue-os/brew:latest"

# Bluefin shared files (ujust, uupd, setup, signing, udev...) and Homebrew
FROM ${COMMON_IMAGE} AS common
FROM ${BREW_IMAGE} AS brew

# Build context: scripts + system files, never copied as-is into the final image
FROM scratch AS ctx
COPY build_files /build_files
COPY --from=common /system_files/shared /system_files/common
COPY --from=common /system_files/bluefin /system_files/common-bluefin
COPY --from=brew /system_files /system_files/brew
# Our own files, applied last so they take precedence
COPY system_files /system_files/cosmic-blue
# Outputs of Containerfile.helpers (build context "helpers", see `just build`)
COPY --from=helpers /theme/cosmic /system_files/cosmic-blue/usr/share/cosmic
COPY --from=helpers /theme/previous /previous_defaults/theme
COPY --from=helpers /initial_setup /initial_setup
COPY --from=helpers /greeter /greeter
COPY --from=helpers /plymouth /system_files/cosmic-blue/usr/share/plymouth/themes/cosmic-blue
COPY --from=helpers /icons /icons
COPY --from=helpers /cursors /system_files/cosmic-blue/usr/share/icons
# Public key used to verify the image signature on update
COPY cosign.pub /system_files/cosmic-blue/etc/pki/containers/cosmic-blue.pub

FROM ${BASE_IMAGE}

ARG IMAGE_NAME="cosmic-blue"
ARG IMAGE_VENDOR="cosmic-blue-os"
ARG UBLUE_IMAGE_TAG="latest"
ARG VERSION=""

RUN --mount=type=bind,from=ctx,source=/,target=/ctx \
    --mount=type=cache,dst=/var/cache \
    --mount=type=cache,dst=/var/log \
    --mount=type=tmpfs,dst=/tmp \
    IMAGE_NAME="${IMAGE_NAME}" IMAGE_VENDOR="${IMAGE_VENDOR}" \
    UBLUE_IMAGE_TAG="${UBLUE_IMAGE_TAG}" VERSION="${VERSION}" \
    /ctx/build_files/build.sh

RUN bootc container lint
