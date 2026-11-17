#!/usr/bin/bash
# Static checks, run on pull requests (.github/workflows/lint.yml) and by
# `just lint`: shellcheck on every shell script of the repository and the
# image (warnings and errors only), actionlint on the workflows when it is
# installed (the workflow runs it separately).
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

if command -v actionlint >/dev/null; then
    SHELLCHECK_OPTS="--severity=warning" actionlint
fi
