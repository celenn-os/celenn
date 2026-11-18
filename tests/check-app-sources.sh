#!/usr/bin/bash
# Checks that the apps and tools the image installs on request still exist
# where they come from. The install-* ujust recipes download too much to run
# in the boot test, so an app renamed or removed from Flathub, or a Homebrew
# formula or cask gone, would only show when a person runs the command.
#
# Reads the image's files (not this checkout's: most install recipes and
# Brewfiles come from projectbluefin/common and ublue-os/brew):
#   - Flatpak: first-boot apps (preinstall.d), Brewfile `flatpak` lines, and
#     the app ids on the flatpak lines and app lists of the ujust recipes
#   - Homebrew: Brewfile `brew`, `cask` and `tap` lines, and the formulas and
#     casks of the install-dev-tools list (60-custom.just)
#
# Usage: tests/check-app-sources.sh <folder with the image's /usr/share>
# Needs: flatpak with the flathub remote (--user), brew.
set -uo pipefail

share="$1/usr/share"
just_dir="${share}/ublue-os/just"
brewfiles=("${share}"/ublue-os/homebrew/*.Brewfile "${share}"/ublue-os/homebrew/*/*.Brewfile)
problems=0
problem() { echo "::error::$*"; problems=$((problems + 1)); }

# Flatpak ids look like org.example.App (3 parts or more). COSMIC's own
# config ids (com.system76.Cosmic*) are settings, not apps.
flatpak_ids() {
    grep -ohE '\b[a-z][a-z0-9_-]*\.[A-Za-z0-9_-]+\.[A-Za-z0-9_.-]*[A-Za-z0-9]\b' \
        | grep -vE '^com\.system76\.Cosmic|\.(desktop|service|json|sh|conf|list|toml|flatpakrepo|filter)$' || true
}
{
    awk '/^\[Flatpak Preinstall / { app = substr($3, 1, length($3) - 1) }
        /^IsRuntime=false/ { print app }' "${share}"/flatpak/preinstall.d/*.preinstall
    sed -n 's/^flatpak "\([^"]*\)".*/\1/p' "${brewfiles[@]}" 2>/dev/null
    grep -hE 'flatpak +(install|info|run|uninstall)|apps=\(|\|flatpak\|' "${just_dir}"/*.just | flatpak_ids
} | sort -u > flatpaks.txt
echo "Flatpak refs to check: $(wc -l < flatpaks.txt)"

flatpak remote-ls --user --all --columns=application flathub | sort -u > flathub.txt
[[ -s flathub.txt ]] || { echo "::error::Could not list Flathub"; exit 1; }
while IFS= read -r ref; do
    if ! grep -qxF "${ref}" flathub.txt; then
        problem "Flathub has no ${ref}"
        continue
    fi
    # Apps only (extensions and runtimes have several branches)
    info="$(flatpak remote-info --user flathub "app/${ref}/x86_64/stable" 2>/dev/null)" || continue
    if grep -qiE '^ *end-of-life' <<<"${info}"; then
        problem "${ref} is end-of-life on Flathub: $(grep -iE '^ *end-of-life' <<<"${info}" | head -n 2 | xargs)"
    fi
done < flatpaks.txt

# Homebrew: taps first, so their formulas and casks are known
{ echo ublue-os/tap; sed -n 's/^tap "\([^"]*\)".*/\1/p' "${brewfiles[@]}" 2>/dev/null; } | sort -u > taps.txt
{
    sed -n 's/^brew "\([^"]*\)".*/formula \1/p' "${brewfiles[@]}" 2>/dev/null
    sed -n 's/^cask "\([^"]*\)".*/cask \1/p' "${brewfiles[@]}" 2>/dev/null
    # install-dev-tools list: "Label|formula|names" and "Label|cask|names"
    grep -hoE '"[^"|]*\|(formula|cask)\|[^"]*"' "${just_dir}/60-custom.just" | tr -d '"' \
        | awk -F'|' '{ n = split($3, names, " "); for (i = 1; i <= n; i++) print $2, names[i] }'
} | sort -u > brew.txt
echo "Homebrew taps: $(xargs < taps.txt)"
echo "Homebrew formulas and casks to check: $(wc -l < brew.txt)"
while IFS= read -r tap; do
    brew tap "${tap}" 2>&1 | tail -n 3 || problem "Homebrew tap ${tap} could not be added"
done < taps.txt
while read -r kind name; do
    # Brewfiles name a tap's formula as owner/tap/name; the casks of the dev
    # tools list come from the ublue-os tap
    error="$(brew info "--${kind}" "${name}" 2>&1 >/dev/null)" && continue
    if [[ "${name}" != */* ]]; then
        error="$(brew info "--${kind}" "ublue-os/tap/${name}" 2>&1 >/dev/null)" && continue
    fi
    problem "Homebrew has no ${kind} ${name}: $(tail -n 2 <<<"${error}" | xargs)"
done < brew.txt

if (( problems > 0 )); then
    echo "::error::${problems} app or tool sources are gone: update the recipes or lists that name them"
    exit 1
fi
echo "All $(wc -l < flatpaks.txt) Flatpak refs and $(wc -l < brew.txt) Homebrew formulas and casks exist"
