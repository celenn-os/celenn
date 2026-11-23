#!/usr/bin/env bash
# Downloads the desktop screenshots of the latest published image for the
# website: the boot test of build.yml (tests/vm-boot-test.sh) logs a test
# user in and takes them. Artifacts last 7 days and main is built every day.
#
# "Published" means the build pushed the image under its own version tag
# (step "Push the version tag"), even if a later step failed: the image
# exists, so the site shows it. Only the newest published build is used. If
# its screenshots cannot be downloaded, this script fails, so the site is
# never built with screenshots older than the latest image.
#
# Usage: website/fetch-screenshots.sh <output directory>
# Writes the screenshots (PNG or PPM) and info.env (date, commit, run URL).
# Needs gh with GH_TOKEN (actions: read) and GITHUB_REPOSITORY.
set -uo pipefail

out="$1"
repo="${GITHUB_REPOSITORY:-cosmic-blue-os/cosmic-blue}"
artifact="boot-test-cosmic-blue"  # boot-test-${IMAGE_NAME} in build.yml
job="Build and push image"         # build_push job of the standard image
tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT
mkdir -p "${out}"

fail() { echo "::error file=website/fetch-screenshots.sh::Screenshots: $*"; exit 1; }

# Finished builds of main, newest first (pull request runs publish nothing)
runs="$(gh api "repos/${repo}/actions/workflows/build.yml/runs?branch=main&status=completed&per_page=30" \
    --jq '.workflow_runs[] | select(.event != "pull_request") | [.id, .head_sha, .created_at, .html_url] | @tsv')" \
    || fail "could not list the image builds"

while IFS=$'\t' read -r id sha created url; do
    [[ -n "${id}" ]] || continue
    # Did this build publish the standard image?
    pushed="$(gh api "repos/${repo}/actions/runs/${id}/jobs?per_page=100" \
        --jq ".jobs[] | select(.name == \"${job}\") | .steps[] | select(.name == \"Push the version tag\") | .conclusion")" \
        || fail "could not read the jobs of ${url}"
    [[ "${pushed}" == success ]] || continue

    echo "Latest published image: ${url} (${sha:0:7}, ${created})"
    gh api "repos/${repo}/actions/runs/${id}/artifacts" \
        --jq ".artifacts[] | select(.name == \"${artifact}\" and (.expired | not)) | .id" \
        | grep -q . || fail "${url} has no ${artifact} artifact (expired?)"
    gh run download "${id}" -R "${repo}" -n "${artifact}" -D "${tmp}" >/dev/null \
        || fail "could not download ${artifact} from ${url}"
    # The boot test logs the test user in before the image is pushed
    compgen -G "${tmp}/screenshots/*-desktop.*" >/dev/null \
        || fail "${url} has no desktop screenshot"
    cp "${tmp}"/screenshots/* "${out}/"
    printf 'SHOTS_DATE=%s\nSHOTS_COMMIT=%s\nSHOTS_RUN=%s\n' \
        "${created%%T*}" "${sha:0:7}" "${url}" > "${out}/info.env"
    ls "${out}"
    exit 0
done <<< "${runs}"

fail "no published image among the last 30 builds of main"
