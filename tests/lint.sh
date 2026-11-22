#!/usr/bin/bash
# Static checks, run on pull requests (.github/workflows/lint.yml) and by
# `just lint`: shellcheck on every shell script of the repository and the
# image (warnings and errors only), rustfmt on the Rust programs, the apps'
# translatable texts (tests/check-translations.py), actionlint on the
# workflows when it is installed (the workflow runs it separately).
set -euo pipefail
cd "$(dirname "$0")/.."

mapfile -t scripts < <(
    git ls-files '*.sh'
    # Scripts without the .sh extension, found by their #! line
    git ls-files -z system_files | while IFS= read -r -d '' file; do
        [[ -f "${file}" && ! -L "${file}" ]] || continue
        head -n 1 "${file}" | grep -qE '^#!.*\b(ba)?sh\b' && echo "${file}"
    done
)
echo "shellcheck: ${#scripts[@]} scripts"
shellcheck --severity=warning "${scripts[@]}"

# Rust: the crate roots (rustfmt follows their modules); no build needed
mapfile -t crates < <(git ls-files 'build_files/*/Cargo.toml')
roots=()
for manifest in "${crates[@]}"; do
    dir="${manifest%/Cargo.toml}"
    while IFS= read -r root; do
        roots+=("${dir}/${root}")
    done < <(sed -n 's/^path = "\(.*\.rs\)"$/\1/p' "${manifest}")
    if [[ -f "${dir}/src/main.rs" ]]; then
        roots+=("${dir}/src/main.rs")
    fi
done
mapfile -t roots < <(printf '%s\n' "${roots[@]}" | sort -u)
echo "rustfmt: ${#roots[@]} crate roots"
rustfmt --edition 2024 --check "${roots[@]}"

# Every tr("...") of the apps is in its program's translation template
tests/check-translations.py

if command -v actionlint >/dev/null; then
    SHELLCHECK_OPTS="--severity=warning" actionlint
fi
