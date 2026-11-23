#!/usr/bin/env bash
# Assemble the website into the given directory (default: _site).
# The logo and the wallpaper come from the repository, so the site always
# shows the current branding. Needs ImageMagick.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
out="${1:-${root}/_site}"

rm -rf "${out}"
mkdir -p "${out}/img"

# Live ISO download: ISO_BASE_URL is the public URL of the R2 bucket. Until
# it is set, link to the live ISO workflow runs (needs a GitHub login).
# One ISO per image: cosmic-blue, and cosmic-blue-nvidia for NVIDIA cards.
iso_name="cosmic-blue-live-amd64.iso"
iso_nvidia_name="cosmic-blue-nvidia-live-amd64.iso"
runs="https://github.com/cosmic-blue-os/cosmic-blue/actions/workflows/build-live-iso.yml?query=branch%3Amain+is%3Asuccess"

# Size of each ISO online (Content-Length), shown next to the download button
declare -A iso_sizes=()
if [[ -n "${ISO_BASE_URL:-}" ]]; then
  for kind in main nvidia; do
    name="${iso_name}"
    [[ "${kind}" == nvidia ]] && name="${iso_nvidia_name}"
    bytes="$(curl -fsSIL --retry 2 --max-time 20 "${ISO_BASE_URL%/}/${name}" 2>/dev/null \
      | tr -d '\r' | awk 'tolower($1) == "content-length:" { n = $2 } END { print n }' || true)"
    if [[ "${bytes}" =~ ^[0-9]+$ ]] && (( bytes > 100000000 )); then
      iso_sizes[${kind}]="$(awk -v b="${bytes}" 'BEGIN { printf "%.1f", b / 1e9 }')"
      echo "${name}: ${iso_sizes[${kind}]} GB"
    fi
  done
fi

# Desktop screenshots for the screenshots page and the home page preview,
# from SCREENSHOTS_DIR (filled by website/fetch-screenshots.sh in CI). Files
# are named like 03-desktop.png; the pages show a screenshot only if it is
# there (blocks between <!-- shot:NAME --> and <!-- /shot -->).
shots_dir="${SCREENSHOTS_DIR:-}"
shots=""
zoom_shots="parental-controls-child parental-more-time first-steps-2 first-steps-3 tools-overview tools-software updates"
if [[ -n "${shots_dir}" && -f "${shots_dir}/info.env" ]]; then
  mkdir -p "${out}/img/screenshots"
  for file in "${shots_dir}"/*.png "${shots_dir}"/*.ppm; do
    [[ -e "${file}" ]] || continue
    name="$(basename "${file%.*}")"
    name="${name#[0-9][0-9]-}"
    [[ " ${shots} " == *" ${name} "* ]] && continue
    convert "${file}" -strip -quality 90 -define webp:method=6 "${out}/img/screenshots/${name}.webp"
    convert "${file}" -resize 960x -strip -quality 85 -define webp:method=6 "${out}/img/screenshots/${name}-960.webp"
    # The "Why cosmic-blue" page shows app windows closer: the middle of the
    # screen, where the window opens
    if [[ " ${zoom_shots} " == *" ${name} "* ]]; then
      convert "${file}" -gravity center -crop 1200x760+0-40 +repage -strip -quality 85 \
        -define webp:method=6 "${out}/img/screenshots/${name}-zoom.webp"
    fi
    shots+=" ${name}"
  done
  # shellcheck source=/dev/null
  source "${shots_dir}/info.env"
elif [[ -n "${shots_dir}" ]]; then
  # CI always passes the screenshots of the latest image
  echo "::error file=website/build.sh::SCREENSHOTS_DIR=${shots_dir} has no screenshots (see website/fetch-screenshots.sh)"
  exit 1
fi
# The "Why cosmic-blue" page needs its screenshots: fail when the boot test
# no longer takes one (renamed or removed) rather than drop it silently
for name in ${zoom_shots}; do
  if ! grep -qE "[0-9]{2}-${name%-[0-9]}" "${root}/tests/vm-lib.sh"; then
    echo "::error file=website/build.sh::screenshot ${name} (zoom_shots) is not taken by tests/vm-lib.sh"
    exit 1
  fi
  if [[ -n "${shots}" && " ${shots} " != *" ${name} "* ]]; then
    echo "::error file=website/build.sh::screenshot ${name} (zoom_shots) is missing from the latest boot test"
    exit 1
  fi
done
# With screenshots (CI), every screenshot a page shows must be in the latest
# boot test: a page never keeps an old one, nor silently drops one
if [[ -n "${shots}" ]]; then
  missing=0
  for name in $(find "${root}/website" -name '*.html' -exec grep -ohE '<!-- shot:[a-z0-9-]+ -->' {} + \
      | sed -E 's/<!-- shot:([a-z0-9-]+) -->/\1/' | sort -u); do
    if [[ " ${shots} " != *" ${name} "* ]]; then
      echo "::error file=website/build.sh::screenshot ${name} is shown on the website but missing from the latest boot test"
      missing=1
    fi
  done
  (( missing == 0 )) || exit 1
fi
[[ -n "${shots}" ]] && shots+=" shots-info" || shots="no-shots"

# The same pages in every language: English at the root (the default), the
# others in their own folder (fr/, de/...).
languages=(en de es fr it pt-br)
pages=(index.html install/index.html docs/index.html docs/troubleshooting/index.html screenshots/index.html why/index.html)
all_pages=()
for lang in "${languages[@]}"; do
  for page in "${pages[@]}"; do
    [[ "${lang}" == en ]] && all_pages+=("${page}") || all_pages+=("${lang}/${page}")
  done
done

# Keep the translations in step with the English pages: same sections, same
# commands, same screenshots, and a link to every language in the menu.
sync_errors=0
outline() {
  grep -oE 'id="[^"]+"|<pre>|<!-- /?(shot(:[a-z0-9-]+)?|shots-info|no-shots|checksum|/checksum) -->|data-(nvidia|main)-only|data-iso-size="[a-z]+"|hreflang="[A-Za-z-]+"' "$1"
}
for page in "${pages[@]}"; do
  for lang in "${languages[@]:1}"; do
    if ! diff <(outline "${root}/website/${page}") <(outline "${root}/website/${lang}/${page}") > /dev/null; then
      echo "::error file=website/${lang}/${page}::not in step with website/${page} (sections, commands, screenshots or language links differ)"
      sync_errors=1
    fi
  done
done
(( sync_errors == 0 )) || exit 1

for page in "${all_pages[@]}"; do
  mkdir -p "$(dirname "${out}/${page}")"
  if [[ -n "${ISO_BASE_URL:-}" ]]; then
    sed -e "s|@ISO_NVIDIA_URL@|${ISO_BASE_URL%/}/${iso_nvidia_name}|g" \
        -e "s|@ISO_URL@|${ISO_BASE_URL%/}/${iso_name}|g" \
      "${root}/website/${page}" > "${out}/${page}"
    # The size of the ISOs online, in GB with one decimal (a decimal comma
    # in every language but English); the size written in the page stays if
    # it cannot be read
    for kind in main nvidia; do
      size="${iso_sizes[${kind}]:-}"
      [[ -n "${size}" ]] || continue
      [[ "${page}" == */* && "${page}" != docs/* && "${page}" != screenshots/* ]] && size="${size/./,}"
      sed -i "s|<span data-iso-size=\"${kind}\">[^<]*</span>|<span data-iso-size=\"${kind}\">${size}</span>|" "${out}/${page}"
    done
  else
    sed -e 's|<!-- checksum -->.*<!-- /checksum -->||' \
        -e "s|@ISO_NVIDIA_URL@|${runs}|g" -e "s|@ISO_URL@|${runs}|g" \
      "${root}/website/${page}" > "${out}/${page}"
  fi
  SHOTS="${shots}" perl -0pi -e '
    my %have = map { $_ => 1 } split " ", $ENV{SHOTS};
    s{^[ \t]*<!-- (?:shot:([\w-]+)|(shots-info|no-shots)) -->.*?<!-- /(?:shot|shots-info|no-shots) -->\n}
     {$have{$1 // $2} ? $& : ""}gmse;
    s{<!-- /?(shot(:[\w-]+)?|shots-info|no-shots) -->}{}g;' "${out}/${page}"
  sed -i -e "s|@SHOTS_DATE@|${SHOTS_DATE:-}|g" -e "s|@SHOTS_COMMIT@|${SHOTS_COMMIT:-}|g" \
    -e "s|@SHOTS_RUN@|${SHOTS_RUN:-}|g" "${out}/${page}"
  # The screenshots keep their names from one build to the next: the commit
  # in their address makes browsers and link previews load the new ones
  if [[ -n "${SHOTS_COMMIT:-}" ]]; then
    sed -i -E "s#(img/screenshots/[a-z0-9-]+\.webp)#\1?v=${SHOTS_COMMIT}#g" "${out}/${page}"
  fi
done
cp "${root}/website/style.css" "${root}/website/site.js" "${out}/"
cp "${root}/branding/logo/cosmic-blue-logo.svg" \
   "${root}/branding/logo/cosmic-blue-logo-horizontal-dark.svg" "${out}/img/"

# Header image: WebP (about a quarter of the JPEG size), JPEG kept for old
# browsers and for link previews (og:image)
wallpaper="${root}/system_files/usr/share/backgrounds/cosmic-blue/cosmic-blue.jpg"
for width in 1280 2560; do
  convert "${wallpaper}" -resize "${width}x" -strip -interlace Plane \
    -sampling-factor 4:2:0 -quality 82 "${out}/img/wallpaper-${width}.jpg"
  convert "${wallpaper}" -resize "${width}x" -strip -quality 80 \
    -define webp:method=6 "${out}/img/wallpaper-${width}.webp"
done

# Every image a page links to must be in the site: screenshots, logo,
# wallpaper (src, srcset, href and og:image addresses)
broken=0
for page in "${all_pages[@]}"; do
  dir="$(dirname "${out}/${page}")"
  while read -r ref; do
    ref="${ref%%\?*}"
    if [[ ! -f "${dir}/${ref}" ]]; then
      echo "::error file=website/${page}::image ${ref} is not in the site"
      broken=1
    fi
  done < <(grep -oE '(src|srcset|href|content)="[^"]+"' "${out}/${page}" \
    | sed -E 's/^[a-z]+="//; s/"$//' | tr ',' '\n' | awk '{ print $1 }' \
    | grep -E '\.(webp|png|jpe?g|svg|gif)(\?|$)' | grep -vE '^(https?:)?//' || true)
  # Absolute addresses of this site (og:image) are checked too
  while read -r ref; do
    ref="${ref%%\?*}"
    [[ -f "${out}/${ref#https://cosmic-blue-os.github.io/cosmic-blue/}" ]] && continue
    echo "::error file=website/${page}::image ${ref} is not in the site"
    broken=1
  done < <(grep -oE 'https://cosmic-blue-os\.github\.io/cosmic-blue/[^" ]+\.(webp|png|jpe?g|svg|gif)' "${out}/${page}" || true)
done
(( broken == 0 )) || exit 1

# Stop GitHub Pages from running Jekyll on the files
touch "${out}/.nojekyll"
