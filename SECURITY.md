# Security policy

## Supported versions

Only the current images get security fixes: `ghcr.io/cosmic-blue-os/cosmic-blue`
and `ghcr.io/cosmic-blue-os/cosmic-blue-nvidia`, tags `stable` and `latest`.
Older tags are not updated: `sudo bootc upgrade` (or the automatic updates)
moves an installed system to the current image.

## Reporting a vulnerability

Please do not open a public issue. Report it privately with GitHub's
[private vulnerability reporting](https://github.com/cosmic-blue-os/cosmic-blue/security/advisories/new)
(Security tab, "Report a vulnerability"). Say what is affected, how to
reproduce it and the image version (`sudo bootc status`).

You should get an answer within a week. Fixed images are built and signed by
CI, and the advisory is published once installed systems can update.

## Scope

In scope: what this repository adds to the image (the Containerfile, build
scripts, services and configuration under `system_files/`), the CI that
builds, signs and publishes the images and live ISOs, and the website.

Problems in the software cosmic-blue ships unchanged (Fedora packages, COSMIC,
the Universal Blue base, Flatpak apps) belong to those projects: please report
them upstream. If you are not sure, report here and we will forward it.
