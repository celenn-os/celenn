set dotenv-filename := "image-template.env"
set dotenv-load

export image_name := env_var("IMAGE_NAME")
export repo_organization := env_var("REPO_ORGANIZATION")
export repo_name := env_var("REPO_NAME")
export image_desc := env_var("IMAGE_DESC")
export image_keywords := env_var("IMAGE_KEYWORDS")
export image_logo_url := env_var("IMAGE_LOGO_URL")
export default_tag := env_var("DEFAULT_TAG")
export bib_image := env_var("BIB_IMAGE")

alias build-vm := build-qcow2
alias rebuild-vm := rebuild-qcow2
alias run-vm := run-vm-qcow2

[private]
default:
    @just --list

# Check Just Syntax
[group('Just')]
check:
    #!/usr/bin/env bash
    find . -type f -name "*.just" | while read -r file; do
    	echo "Checking syntax: $file"
    	just --unstable --fmt --check -f $file
    done
    echo "Checking syntax: Justfile"
    just --unstable --fmt --check -f Justfile

# Fix Just Syntax
[group('Just')]
fix:
    #!/usr/bin/env bash
    find . -type f -name "*.just" | while read -r file; do
    	echo "Checking syntax: $file"
    	just --unstable --fmt -f $file
    done
    echo "Checking syntax: Justfile"
    just --unstable --fmt -f Justfile || { exit 1; }

# Clean Repo
[group('Utility')]
clean:
    #!/usr/bin/env bash
    set -eoux pipefail
    touch _build
    find *_build* -exec rm -rf {} \;
    rm -f previous.manifest.json
    rm -f changelog.md
    rm -f output.env
    rm -rf output/

# Sudo Clean Repo
[group('Utility')]
[private]
sudo-clean:
    just sudoif just clean

# sudoif bash function
[group('Utility')]
[private]
sudoif command *args:
    #!/usr/bin/env bash
    function sudoif(){
        if [[ "${UID}" -eq 0 ]]; then
            "$@"
        elif [[ "$(command -v sudo)" && -n "${SSH_ASKPASS:-}" ]] && [[ -n "${DISPLAY:-}" || -n "${WAYLAND_DISPLAY:-}" ]]; then
            sudo --askpass "$@" || exit 1
        elif [[ "$(command -v sudo)" ]]; then
            sudo "$@" || exit 1
        else
            exit 1
        fi
    }
    sudoif {{ command }} {{ args }}

# This Justfile recipe builds a container image using Podman.
#
# Arguments:
#   $target_image - The tag you want to apply to the image (default: $image_name).
#   $tag - The tag for the image (default: $default_tag).
#
#   $version - The image version (see the `version` recipe). Default: the
#              Fedora release and today's date, without looking at the registry.
#
# If the git working directory is clean, the labels also point to the commit.
#
# just build $target_image $tag $version
#
# Example usage:
#   just build myimage mytag
#
# This will build an image 'myimage:mytag'
#

# Build the image using the specified parameters
build $target_image=image_name $tag=default_tag $version="":
    #!/usr/bin/env bash

    set -euox pipefail

    if [[ -z "${version}" ]]; then
        version="$(just fedora-version).$(date -u +%Y%m%d)"
    fi

    # The image name selects the variant (e.g. cosmic-blue-nvidia)
    BUILD_ARGS=("--build-arg" "IMAGE_NAME=${target_image}" "--build-arg" "VERSION=${version}")
    # Read by bootc (bootc status) and written to the aleph file by bootc install
    LABELS=("--label" "org.opencontainers.image.version=${version}")
    if [[ -z "$(git status -s)" ]]; then
        GIT_SHA=$(git rev-parse --short HEAD)
        # The commit the image was built from (read by release-notes.yml)
        LABELS+=("--label" "org.opencontainers.image.revision=$(git rev-parse HEAD)")
        LABELS+=("--label" "io.artifacthub.package.readme-url=https://raw.githubusercontent.com/{{ repo_organization }}/{{ repo_name }}/${GIT_SHA}/README.md")
        LABELS+=("--label" "org.opencontainers.image.documentation=https://raw.githubusercontent.com/{{ repo_organization }}/{{ repo_name }}/${GIT_SHA}/README.md")
        LABELS+=("--label" "org.opencontainers.image.source=https://github.com/{{ repo_organization }}/{{ repo_name }}/blob/${GIT_SHA}/Containerfile")
        LABELS+=("--label" "org.opencontainers.image.url=https://github.com/{{ repo_organization }}/{{ repo_name }}/tree/${GIT_SHA}")
    fi

    # Image metadata for https://artifacthub.io/ - This is optional but is highly recommended so we all can get a index of all the custom images
    # The metadata by itself is not going to do anything, you choose if you want your image to be on ArtifactHub or not.
    LABELS+=("--label" "io.artifacthub.package.deprecated=false")
    LABELS+=("--label" "io.artifacthub.package.keywords={{ image_keywords }}")
    LABELS+=("--label" "io.artifacthub.package.license=Apache-2.0")
    LABELS+=("--label" "io.artifacthub.package.logo-url={{ image_logo_url }}")
    LABELS+=("--label" "io.artifacthub.package.prerelease=false")
    LABELS+=("--label" "org.opencontainers.image.created=$(date -u +%Y\-%m\-%d\T%H\:%M\:%S\Z)")
    LABELS+=("--label" "org.opencontainers.image.description={{ image_desc }}")
    LABELS+=("--label" "org.opencontainers.image.title=${target_image}")
    LABELS+=("--label" "org.opencontainers.image.vendor={{ repo_organization }}")

    # The helper stages (Containerfile.helpers): already built in HELPERS_DIR
    # (e.g. restored from the CI cache), or built now
    if [[ -z "${HELPERS_DIR:-}" ]]; then
        HELPERS_DIR="$(mktemp -d)"
        trap 'rm -rf "${HELPERS_DIR}"' EXIT
        just build-helpers "${HELPERS_DIR}"
    fi
    test -d "${HELPERS_DIR}/theme"

    # This actually builds the image!
    PODMAN_BUILD_ARGS=("${BUILD_ARGS[@]}" "${LABELS[@]}" --pull=newer --tag "${target_image}:${tag}" --file Containerfile)
    PODMAN_BUILD_ARGS+=(--build-context "helpers=${HELPERS_DIR}")

    podman build "${PODMAN_BUILD_ARGS[@]}" .

# Helper stages of Containerfile.helpers, in build order
helper-stages:
    @echo theme tour greeter store filemanager plymouth icons cursors

# Build helper stages of Containerfile.helpers (all of them, or the ones
# given) and write the output of each one to <dest>/<stage>, used by `just
# build` as the "helpers" build context
build-helpers $dest *stages:
    #!/usr/bin/env bash
    set -euxo pipefail

    stages="{{ stages }}"
    [[ -n "${stages}" ]] || stages="$(just helper-stages)"
    base_tag="$(just fedora-version)"
    for stage in ${stages}; do
        rm -rf "${dest:?}/${stage}"
        mkdir -p "${dest}/${stage}"
        # Rust stages: compiled dependencies kept in HELPERS_CARGO_CACHE/<stage>
        # if set (see Containerfile.helpers)
        volume=()
        if [[ -n "${HELPERS_CARGO_CACHE:-}" ]] && grep -qE "^ENV CARGO_(HOME|CACHE_DIR)=/cargo" \
                <(awk -v build="AS ${stage}-build" 'index($0, build) { on = 1 } on && /^FROM / && !index($0, build) { exit } on' Containerfile.helpers); then
            mkdir -p "${HELPERS_CARGO_CACHE}/${stage}"
            volume=(--volume "$(realpath "${HELPERS_CARGO_CACHE}/${stage}"):/cargo")
        fi
        podman build --pull=newer --build-arg "BASE_TAG=${base_tag}" "${volume[@]}" \
            --file Containerfile.helpers --target "${stage}" \
            --output "type=local,dest=${dest}/${stage}" .
    done

# Cache keys of the helper stages, one "<stage> <key>" line each. A stage's
# key changes with its own part of Containerfile.helpers, the files it copies
# and what it builds from outside the repository: the Fedora release, the
# Fedora package it rebuilds or recolors (Fedora updates them without any
# change here), the Rust image of the theme stage. Not the digest of the
# Fedora image, rebuilt every few days for the same result; the tour, which
# links to Fedora's libraries without a package of its own to follow, is
# rebuilt once a week instead.
helpers-keys:
    #!/usr/bin/env bash
    set -euo pipefail

    base_tag="$(just fedora-version)"
    fedora="registry.fedoraproject.org/fedora:${base_tag}"
    # Fedora packages: source packages rebuilt, Plymouth spinner recolored,
    # and cosmic-settings for the COSMIC version
    packages="$(podman run --rm --pull=newer "${fedora}" \
        dnf5 -q repoquery --enablerepo=fedora-source --enablerepo=updates-source \
        --latest-limit=1 --queryformat '%{full_nevra}\n' \
        cosmic-files cosmic-greeter cosmic-settings cosmic-store plymouth-theme-spinner | sort)"
    # COSMIC must be the version cosmic-blue is tested against
    # (build_files/cosmic-version): a new one first goes through the pull
    # request of check-cosmic-version.yml, which moves that file. Checked here,
    # before the helper stages that patch COSMIC programs, so the build stops
    # with this message rather than a patch that no longer applies.
    pinned="$(grep -v '^#' build_files/cosmic-version | tr -d '[:space:]')"
    fedora_cosmic="$(sed -nE 's/^cosmic-settings-([0-9]+:)?([^-]+)-.*/\2/p' <<< "${packages}" | sort -u)"
    if [[ "${fedora_cosmic}" != "${pinned}" ]]; then
        echo "::error::Fedora ${base_tag} now ships COSMIC ${fedora_cosmic:-?}, but cosmic-blue is pinned to ${pinned} (build_files/cosmic-version): merge the COSMIC update pull request of check-cosmic-version.yml first" >&2
        exit 1
    fi
    package() {
        # Found, or a stale cache could be used
        grep "^${1}-[0-9]" <<< "${packages}"
    }
    files() {
        git ls-files -z "$@" | sort -z | xargs -0 sha256sum
    }
    inputs="$(mktemp)"
    trap 'rm -f "${inputs}"' EXIT
    for stage in $(just helper-stages); do
        {
            echo "stage ${stage}, fedora ${base_tag}"
            awk -v build="AS ${stage}-build" -v copy="COPY --from=${stage}-build /out /" \
                'index($0, build) { on = 1 } on { print } on && $0 == copy { exit }' \
                Containerfile.helpers
            case "${stage}" in
                theme)
                    files build_files/theme
                    rust=docker.io/library/rust:1
                    echo "${rust} $(skopeo inspect --retry-times 3 --format '{{ '{{.Digest}}' }}' "docker://${rust}")" ;;
                tour)
                    files build_files/first_steps
                    echo "week $(date -u +%G-W%V)" ;;
                greeter)
                    files build_files/greeter system_files/usr/share/backgrounds/cosmic-blue/cosmic-blue-arcs.jpg
                    package cosmic-greeter ;;
                store)
                    files build_files/store
                    package cosmic-store ;;
                filemanager)
                    files build_files/filemanager
                    package cosmic-files ;;
                plymouth)
                    files build_files/plymouth
                    package plymouth-theme-spinner ;;
                icons)
                    files build_files/icon_theme/tela.sh ;;
                cursors)
                    files build_files/cursor_theme ;;
                *)
                    echo "No cache key inputs for the helper stage ${stage}" >&2
                    exit 1 ;;
            esac
        } > "${inputs}"
        # The inputs, for the CI log
        sed "s/^/${stage}: /" "${inputs}" >&2
        echo "${stage} $(sha256sum < "${inputs}" | cut -c1-32)"
    done

# Split the image for smaller updates (New)!
rechunk $target_image=image_name $tag=default_tag:
    #!/usr/bin/env bash

    set -xeuo pipefail

    # TODO: pin chunkah image to hash once mature enough
    # You may run into space issues on github runners as we are making a
    # complete copy of the image, which likely has no shared layers, unless your
    # base image is also using chunkah
    CHUNKAH_CONFIG_FILE="$(mktemp)"

    # You may omit the current directory here if you are confident that you
    # won't run out of space on /tmp for your image
    CHUNKAH_OUTPUT_DIR="$(mktemp -d ./"${target_image}"_chunkah_XXXXXX)"

    trap 'rm -f "${CHUNKAH_CONFIG_FILE}"; rm -rf "${CHUNKAH_OUTPUT_DIR}"' EXIT
    podman inspect "${target_image}:${tag}" > "${CHUNKAH_CONFIG_FILE}"

    podman run --rm \
      --mount=type=image,src="${target_image}:${tag}",target=/chunkah \
      -v "${CHUNKAH_CONFIG_FILE}:/chunkah-config.json:ro,Z" \
      -v "${CHUNKAH_OUTPUT_DIR}:/run/out:Z" \
      quay.io/coreos/chunkah:latest \
      build \
      --verbose \
      --compressed \
      --max-layers 128 \
      --prune /sysroot/ \
      --label ostree.commit- --label ostree.final-diffid- \
      --config /chunkah-config.json \
      --output oci:/run/out/chunked

    CHUNKED_IMAGE="$(podman pull "oci:${CHUNKAH_OUTPUT_DIR}/chunked")"
    podman tag "${CHUNKED_IMAGE}" "${target_image}:${tag}"

# Split the image for smaller updates (Classical)!
ostree-rechunk $target_image=image_name $tag=default_tag:
    #!/usr/bin/env bash

    set -xeuo pipefail

    # Use the already-built local image to avoid pulling from a remote registry
    RPM_OSTREE_CHUNKER_IMAGE="localhost/${target_image}:${tag}"

    # The chunked image gets a new config without our labels; keep the OCI and
    # Artifact Hub ones (not the ostree ones, which describe the old image), in
    # particular org.opencontainers.image.version (without it `bootc install`
    # writes a null version that bootupd fails to read on every boot).
    LABEL_ARGS=()
    while IFS= read -r label; do
        LABEL_ARGS+=("--label" "${label}")
    done < <(podman inspect "${RPM_OSTREE_CHUNKER_IMAGE}" \
        | jq -r '.[0].Config.Labels // {} | to_entries[]
                 | select(.key | test("^(org\\.opencontainers\\.image|io\\.artifacthub)\\."))
                 | "\(.key)=\(.value)"')

    GRAPHROOT="$(podman info --format '{{ '{{.Store.GraphRoot}}' }}')"

    podman run --rm --pull=never --privileged \
      --mount=type=image,src="${target_image}:${tag}",target=/rpm-ostree \
      --mount=type=bind,src=${GRAPHROOT},target=/run/host-container-storage,rw \
      --mount=type=tmpfs,target=/run/rpm-ostree-storage \
      --entrypoint /usr/bin/rpm-ostree \
      "${RPM_OSTREE_CHUNKER_IMAGE}" \
      compose build-chunked-oci \
      --max-layers 127 \
      --format-version=2 \
      --bootc \
      --rootfs /rpm-ostree \
      --output "containers-storage:[overlay@/run/host-container-storage+/run/rpm-ostree-storage]localhost/${target_image}:${tag}"

    if (( ${#LABEL_ARGS[@]} > 0 )); then
        LABEL_CTX="$(mktemp -d)"
        echo "FROM localhost/${target_image}:${tag}" > "${LABEL_CTX}/Containerfile"
        podman build --pull=never "${LABEL_ARGS[@]}" --tag "localhost/${target_image}:${tag}" "${LABEL_CTX}"
        rm -rf "${LABEL_CTX}"
    fi

# Fedora release of the image (BASE_TAG in the Containerfile)
[group('Utility')]
fedora-version:
    #!/usr/bin/env bash
    set -euo pipefail

    sed -n 's/^ARG BASE_TAG="\([0-9]*\)"$/\1/p' Containerfile | grep -x '[0-9][0-9]*'

# Version of the next published image, as in Bluefin: <Fedora>.<YYYYMMDD>
# (UTC), then .1, .2... for the later builds of the same day. A version is
# taken if any of the given images (default: the main one) has it as a tag
# on the registry, so that all variants built together share one version.
[group('Utility')]
version *images:
    #!/usr/bin/env bash
    set -euo pipefail

    base="$(just fedora-version).$(date -u +%Y%m%d)"
    images=({{ images }})
    if (( ${#images[@]} == 0 )); then
        images=("{{ image_name }}")
    fi

    tags="$(mktemp)"
    trap 'rm -f "${tags}"' EXIT
    for image in "${images[@]}"; do
        ref="docker://ghcr.io/{{ lowercase(repo_organization) }}/${image,,}"
        if ! out="$(skopeo list-tags --retry-times 3 "${ref}" 2>&1)"; then
            # A repository that was never pushed has no tags; any other error
            # could hide a taken version, so it stops the build.
            if grep -qiE 'name unknown|repository name not known|not found' <<< "${out}"; then
                echo "No tags yet for ${ref}" >&2
                continue
            fi
            echo "${out}" >&2
            exit 1
        fi
        jq -r '.Tags[]' <<< "${out}" >> "${tags}"
    done

    version="${base}"
    point=0
    while grep -qxF "${version}" "${tags}"; do
        point=$((point + 1))
        version="${base}.${point}"
    done
    echo "${version}"

# Generate Default Tag
[group('Utility')]
generate-default-tag $tag=default_tag:
    #!/usr/bin/env bash
    set -eoux pipefail

    echo "${tag}"

# Generate Tags
[group('Utility')]
generate-build-tags $target_image=image_name $tag=default_tag $version="":
    #!/usr/bin/env bash
    set -eoux pipefail

    # The first tag is the build's own (the CI pushes and signs it first):
    # testing: its version (44.20261113.1), then the date (20261113, the last
    # testing build of that day, promoted to stable a week later by
    # promote-stable.yml), testing, and latest (the systems installed before
    # the testing channel follow it). dev: dev-<commit>, then dev.
    case "${tag}" in
        testing)
            [[ -n "${version}" ]] || { echo "testing needs a version" >&2; exit 1; }
            # The date of the version (44.20261113.1 -> 20261113), so that
            # both stay the same when a build runs past midnight
            DATE="$(cut -d. -f2 <<< "${version}")"
            BUILD_TAGS=("${version}" "${DATE}" testing latest)
            ;;
        dev)
            BUILD_TAGS=("dev-$(git rev-parse --short HEAD)" dev)
            ;;
        *)
            # Local builds
            BUILD_TAGS=("${tag}")
            ;;
    esac

    echo "${BUILD_TAGS[@]}"

# Tag Images
[group('Utility')]
tag-images $target_image=image_name $tag=default_tag tags="":
    #!/usr/bin/env bash
    set -eoux pipefail

    # Get Image, and untag
    IMAGE=$(podman inspect ${target_image}:${tag} | jq -r .[].Id)
    podman untag ${IMAGE}

    # Tag Image
    for tag in {{ tags }}; do
        podman tag $IMAGE "${target_image}:${tag}"
    done

    # Show Images
    podman images

# Image Name
[group('Utility')]
[private]
image_name $target_image=image_name:
    #!/usr/bin/env bash
    set -eoux pipefail

    echo "${image_name}"

# Command: _rootful_load_image
# Description: This script checks if the current user is root or running under sudo. If not, it attempts to resolve the image tag using podman inspect.
#              If the image is found, it loads it into rootful podman. If the image is not found, it pulls it from the repository.
#
# Parameters:
#   $target_image - The name of the target image to be loaded or pulled.
#   $tag - The tag of the target image to be loaded or pulled. Default is 'default_tag'.
#
# Example usage:
#   _rootful_load_image my_image latest
#
# Steps:
# 1. Check if the script is already running as root or under sudo.
# 2. Check if target image is in the non-root podman container storage)
# 3. If the image is found, load it into rootful podman using podman scp.
# 4. If the image is not found, pull it from the remote repository into reootful podman.

_rootful_load_image $target_image=image_name $tag=default_tag:
    #!/usr/bin/env bash
    set -eoux pipefail

    # Check if already running as root or under sudo
    if [[ -n "${SUDO_USER:-}" || "${UID}" -eq "0" ]]; then
        echo "Already root or running under sudo, no need to load image from user podman."
        exit 0
    fi

    # Try to resolve the image tag using podman inspect
    set +e
    resolved_tag=$(podman inspect -t image "${target_image}:${tag}" | jq -r '.[].RepoTags.[0]')
    return_code=$?
    set -e

    USER_IMG_ID=$(podman images --filter reference="${target_image}:${tag}" --format "'{{ '{{.ID}}' }}'")

    if [[ $return_code -eq 0 ]]; then
        # If the image is found, load it into rootful podman
        ID=$(just sudoif podman images --filter reference="${target_image}:${tag}" --format "'{{ '{{.ID}}' }}'")
        if [[ "$ID" != "$USER_IMG_ID" ]]; then
            # If the image ID is not found or different from user, copy the image from user podman to root podman
            COPYTMP=$(mktemp -p "${PWD}" -d -t _build_podman_scp.XXXXXXXXXX)
            just sudoif TMPDIR=${COPYTMP} podman image scp ${UID}@localhost::"${target_image}:${tag}" root@localhost::"${target_image}:${tag}"
            rm -rf "${COPYTMP}"
        fi
    else
        # If the image is not found, pull it from the repository
        just sudoif podman pull "${target_image}:${tag}"
    fi

# Build a bootc bootable image using Bootc Image Builder (BIB)
# Converts a container image to a bootable image
# Parameters:
#   target_image: The name of the image to build (ex. localhost/fedora)
#   tag: The tag of the image to build (ex. latest)
#   type: The type of image to build (ex. qcow2, raw, iso)
#   config: The configuration file to use for the build (default: disk_config/disk.toml)

# Example: just _rebuild-bib localhost/fedora latest qcow2 disk_config/disk.toml
_build-bib $target_image $tag $type $config: (_rootful_load_image target_image tag)
    #!/usr/bin/env bash
    set -euo pipefail

    args="--type ${type} "
    args+="--use-librepo=True "
    args+="--rootfs=btrfs"

    BUILDTMP=$(mktemp -p "${PWD}" -d -t _build-bib.XXXXXXXXXX)

    sudo podman run \
      --rm \
      -it \
      --privileged \
      --pull=newer \
      --net=host \
      --security-opt label=type:unconfined_t \
      -v $(pwd)/${config}:/config.toml:ro \
      -v $BUILDTMP:/output \
      -v /var/lib/containers/storage:/var/lib/containers/storage \
      "${bib_image}" \
      ${args} \
      "${target_image}:${tag}"

    mkdir -p output
    sudo mv -f $BUILDTMP/* output/
    sudo rmdir $BUILDTMP
    sudo chown -R $USER:$USER output/

# Podman builds the image from the Containerfile and creates a bootable image
# Parameters:
#   target_image: The name of the image to build (ex. localhost/fedora)
#   tag: The tag of the image to build (ex. latest)
#   type: The type of image to build (ex. qcow2, raw, iso)
#   config: The configuration file to use for the build (deafult: disk_config/disk.toml)

# Example: just _rebuild-bib localhost/fedora latest qcow2 disk_config/disk.toml
_rebuild-bib $target_image $tag $type $config: (build target_image tag) && (_build-bib target_image tag type config)

# Build a QCOW2 virtual machine image
[group('Build Virtal Machine Image')]
build-qcow2 $target_image=("localhost/" + image_name) $tag=default_tag: && (_build-bib target_image tag "qcow2" "disk_config/disk.toml")

# Build a RAW virtual machine image
[group('Build Virtal Machine Image')]
build-raw $target_image=("localhost/" + image_name) $tag=default_tag: && (_build-bib target_image tag "raw" "disk_config/disk.toml")

# Build an ISO virtual machine image
[group('Build Virtal Machine Image')]
build-iso $target_image=("localhost/" + image_name) $tag=default_tag: && (_build-bib target_image tag "iso" "disk_config/iso.toml")

# Rebuild a QCOW2 virtual machine image
[group('Build Virtal Machine Image')]
rebuild-qcow2 $target_image=("localhost/" + image_name) $tag=default_tag: && (_rebuild-bib target_image tag "qcow2" "disk_config/disk.toml")

# Rebuild a RAW virtual machine image
[group('Build Virtal Machine Image')]
rebuild-raw $target_image=("localhost/" + image_name) $tag=default_tag: && (_rebuild-bib target_image tag "raw" "disk_config/disk.toml")

# Rebuild an ISO virtual machine image
[group('Build Virtal Machine Image')]
rebuild-iso $target_image=("localhost/" + image_name) $tag=default_tag: && (_rebuild-bib target_image tag "iso" "disk_config/iso.toml")

# Run a virtual machine with the specified image type and configuration
_run-vm $target_image $tag $type $config:
    #!/usr/bin/env bash
    set -eoux pipefail

    # Determine the image file based on the type
    image_file="output/${type}/disk.${type}"
    if [[ $type == iso ]]; then
        image_file="output/bootiso/install.iso"
    fi

    # Build the image if it does not exist
    if [[ ! -f "${image_file}" ]]; then
        just "build-${type}" "$target_image" "$tag"
    fi

    # Determine an available port to use
    port=8006
    while grep -q :${port} <<< $(ss -tunalp); do
        port=$(( port + 1 ))
    done
    echo "Using Port: ${port}"
    echo "Connect to http://localhost:${port}"

    # Set up the arguments for running the VM
    run_args=()
    run_args+=(--rm --privileged)
    run_args+=(--pull=newer)
    run_args+=(--publish "127.0.0.1:${port}:8006")
    run_args+=(--env "CPU_CORES=4")
    run_args+=(--env "RAM_SIZE=8G")
    run_args+=(--env "DISK_SIZE=64G")
    run_args+=(--env "TPM=Y")
    run_args+=(--env "GPU=Y")
    run_args+=(--device=/dev/kvm)
    run_args+=(--volume "${PWD}/${image_file}":"/boot.${type}")
    run_args+=(docker.io/qemux/qemu)

    # Run the VM and open the browser to connect
    (sleep 30 && xdg-open http://localhost:"$port") &
    podman run "${run_args[@]}"

# Run a virtual machine from a QCOW2 image
[group('Run Virtal Machine')]
run-vm-qcow2 $target_image=("localhost/" + image_name) $tag=default_tag: && (_run-vm target_image tag "qcow2" "disk_config/disk.toml")

# Run a virtual machine from a RAW image
[group('Run Virtal Machine')]
run-vm-raw $target_image=("localhost/" + image_name) $tag=default_tag: && (_run-vm target_image tag "raw" "disk_config/disk.toml")

# Run a virtual machine from an ISO
[group('Run Virtal Machine')]
run-vm-iso $target_image=("localhost/" + image_name) $tag=default_tag: && (_run-vm target_image tag "iso" "disk_config/iso.toml")

# Run a virtual machine using systemd-vmspawn
[group('Run Virtal Machine')]
spawn-vm rebuild="0" type="qcow2" ram="6G":
    #!/usr/bin/env bash

    set -euo pipefail

    [ "{{ rebuild }}" -eq 1 ] && echo "Rebuilding the ISO" && just build-vm {{ rebuild }} {{ type }}

    systemd-vmspawn \
      -M "bootc-image" \
      --console=gui \
      --cpus=2 \
      --ram=$(echo {{ ram }}| numfmt --from=iec) \
      --network-user-mode \
      --vsock=false --pass-ssh-key=false \
      -i ./output/**/*.{{ type }}

# Runs shellcheck on the shell scripts and actionlint on the workflows
lint:
    tests/lint.sh

# Runs shfmt on all Bash scripts
format:
    #!/usr/bin/env bash
    set -eoux pipefail
    # Check if shfmt is installed
    if ! command -v shfmt &> /dev/null; then
        echo "shfmt could not be found. Please install it."
        exit 1
    fi
    # Run shfmt on all Bash scripts
    find . -iname "*.sh" -type f -exec shfmt --write "{}" ';'
