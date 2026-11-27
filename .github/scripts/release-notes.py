#!/usr/bin/env python3
"""Write the release notes of a stable image in Markdown.

Compares the package lists of the previous and the new stable image
(`rpm -qa --qf '%{NAME}\t%{EVR}\t%{SOURCERPM}\n'`, one file each) and lists
the Celenn OS commits between the two builds, sorted by what they change:
the image (or the live ISO), the website and documentation, or only the
build and tests.

It also writes the "Versions" block of README.md (main components of the
new image) to the file given with --versions-block.

Usage: release-notes.py --new new.tsv [--old old.tsv] --title TITLE
       [--image REF] [--commits commits.tsv] [--compare-url URL]
       [--previous-release TAG] [--site URL] [--nvidia EVR]
       [--versions-block FILE --version VERSION]

commits.tsv: one commit per line, oldest first:
SHA <tab> author login <tab> subject <tab> space-separated changed files
"""
import argparse
import re
import sys
from collections import defaultdict
from urllib.parse import quote

# Main components: badges at the top of the notes and in README.md
# (label, package)
VERSIONS = [
    ("Fedora", "fedora-release-common"), ("Kernel", "kernel"),
    ("COSMIC", "cosmic-settings"), ("Mesa", "mesa-dri-drivers"),
]
# "Main packages" table: (label, package). NVIDIA comes from --nvidia.
MAIN_PACKAGES = [
    ("COSMIC", "cosmic-settings"), ("Kernel", "kernel"),
    ("Mesa", "mesa-dri-drivers"), ("NVIDIA driver", None),
    ("Firmware", "linux-firmware"), ("systemd", "systemd"),
    ("bootc", "bootc"), ("rpm-ostree", "rpm-ostree"), ("Flatpak", "flatpak"),
    ("Podman", "podman"), ("PipeWire", "pipewire"),
    ("NetworkManager", "NetworkManager"),
]
BADGE_COLOR = "4f8dff"
# Not packages: imported signing keys
IGNORED = {"gpg-pubkey", "gpg-pubkey-release"}
# Commits that change what is installed: the image or the live ISO
# (Containerfile.helpers alone is mostly build caching)
IMAGE_PATHS = re.compile(
    r"^(Containerfile$|build_files/|system_files/|branding/|cosign\.pub$"
    r"|image-template\.env$|iso_files/|disk_config/"
    r"|\.github/workflows/build-live-iso\.yml$)")
# Commits that only change the website or the documentation
SITE_PATHS = re.compile(r"^(website/|docs/|README\.md$|CONTRIBUTING\.md$|SECURITY\.md$)")
# Bots whose commits are always maintenance (Renovate's base image and
# digest updates still go under "What's new" when they change the image)
README_BOT = "github-actions[bot]"
# GitHub release bodies are limited to 125000 characters
MAX_LENGTH = 120000
STATUS = {"added": "✨", "updated": "🔄", "removed": "❌"}


def read_packages(path):
    """{name: (evr, source package)} from an rpm -qa listing."""
    evrs, sources = defaultdict(set), {}
    with open(path, encoding="utf-8") as f:
        for line in f:
            parts = line.rstrip("\n").split("\t")
            if len(parts) < 2 or not parts[0] or parts[0] in IGNORED:
                continue
            evrs[parts[0]].add(parts[1])
            # kernel-7.3.4-200.fc44.src.rpm -> kernel
            src = parts[2] if len(parts) > 2 else ""
            sources[parts[0]] = src.rsplit("-", 2)[0] if src.count("-") >= 2 else parts[0]
    return {name: (", ".join(sorted(v)), sources[name]) for name, v in evrs.items()}


def evr_of(packages, name):
    return packages[name][0] if name in packages else None


def short(evr):
    """Version for people: no epoch, no Fedora dist tag (1:26.2.7-2.fc44 -> 26.2.7-2)."""
    if not evr:
        return None
    return ", ".join(re.sub(r"\.fc\d+$", "", v.split(":")[-1]) for v in evr.split(", "))


def upstream(evr):
    """Upstream version from an EVR (without epoch and release)."""
    if not evr:
        return None
    return evr.split(", ")[0].split(":")[-1].rsplit("-", 1)[0]


def badge(label, value):
    """shields.io badge in the Celenn OS accent color."""
    def escape(text):
        return quote(text.replace("-", "--").replace("_", "__"), safe="")
    return (f"![{label} {value}](https://img.shields.io/badge/"
            f"{escape(label)}-{escape(value)}-{BADGE_COLOR})")


def change_groups(old, new):
    """Package changes, one row per source package and change:
    [(status, name, count, before, after)], sorted by status then name."""
    groups = defaultdict(list)
    for name in set(old) | set(new):
        before, after = evr_of(old, name), evr_of(new, name)
        if before == after:
            continue
        status = "added" if before is None else "removed" if after is None else "updated"
        source = (new.get(name) or old.get(name))[1]
        groups[(status, source, short(before), short(after))].append(name)
    order = list(STATUS)
    rows = []
    for (status, source, before, after), names in groups.items():
        # A single binary package keeps its own name
        label = names[0] if len(names) == 1 else source
        rows.append((status, label, len(names), before, after))
    return sorted(rows, key=lambda r: (order.index(r[0]), r[1].lower()))


def main_table(old, new, nvidia_old, nvidia_new):
    lines = ["| | Package | Version |", "| :-: | --- | --- |"]
    for label, name in MAIN_PACKAGES:
        if name is None:
            before, after = short(nvidia_old), short(nvidia_new)
        else:
            before, after = short(evr_of(old, name)), short(evr_of(new, name))
        if name == "cosmic-settings":
            # COSMIC is released as a whole: its upstream version is enough
            before, after = upstream(evr_of(old, name)), upstream(evr_of(new, name))
        if not after:
            continue
        if old and before != after:
            icon = STATUS["added"] if before is None else STATUS["updated"]
            version = f"{before} ➡️ **{after}**" if before else f"**{after}**"
            lines.append(f"| {icon} | **{label}** | {version} |")
        else:
            lines.append(f"| | {label} | {after} |")
    return lines


def classify(commits):
    """Sort commits into (image, site, maintenance) lists of (sha, subject)."""
    image, site, maintenance = [], [], []
    for sha, author, subject, files in commits:
        files = files.split()
        if author == README_BOT or subject.startswith("Update README versions"):
            maintenance.append((sha, subject))
        elif any(IMAGE_PATHS.match(f) for f in files):
            image.append((sha, subject))
        elif files and all(SITE_PATHS.match(f) for f in files):
            site.append((sha, subject))
        else:
            maintenance.append((sha, subject))
    return image, site, maintenance


def commit_line(sha, subject):
    """'Subject (#141)' stays as is (GitHub links the number), otherwise the
    short commit hash is added (linked too)."""
    if re.search(r"\(#\d+\)$", subject):
        return f"- {subject}"
    return f"- {subject} ({sha[:7]})"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--new", required=True)
    parser.add_argument("--old")
    parser.add_argument("--title", required=True)
    parser.add_argument("--image", help="Image reference of this exact build")
    parser.add_argument("--commits")
    parser.add_argument("--compare-url")
    parser.add_argument("--previous-release", help="Tag of the previous stable release")
    parser.add_argument("--site", help="Website URL")
    parser.add_argument("--nvidia", help="EVR of the NVIDIA driver in the NVIDIA image")
    parser.add_argument("--nvidia-old", help="EVR of the NVIDIA driver before")
    parser.add_argument("--versions-block", help="Write the README versions block here")
    parser.add_argument("--version", help="Celenn OS version, for the versions block")
    args = parser.parse_args()

    new = read_packages(args.new)
    if not new:
        sys.exit(f"No packages in {args.new}")
    old = read_packages(args.old) if args.old else {}

    versions = [(label, upstream(evr_of(new, name))) for label, name in VERSIONS]
    versions.append(("NVIDIA", upstream(args.nvidia)))
    badges = " ".join(badge(label, value) for label, value in versions if value)

    if args.versions_block:
        about = f" (Celenn OS {args.version})" if args.version else ""
        block = [f"In the current `stable` image{about}:", "",
                 # No badge for a version not found (e.g. no NVIDIA image)
                 "\n".join(badge(label, value) for label, value in versions if value)]
        with open(args.versions_block, "w", encoding="utf-8") as f:
            f.write("\n".join(block) + "\n")

    out = [f"# {args.title}", "", badges, ""]
    since = f" since `{args.previous_release}`" if args.previous_release else ""
    out += [f"The weekly stable update of Celenn OS{since}. Installed systems "
            "download it on their own: it is used after the next reboot.", ""]

    commits = []
    if args.commits:
        with open(args.commits, encoding="utf-8") as f:
            for line in f:
                if line.strip():
                    parts = line.rstrip("\n").split("\t")
                    commits.append((parts + ["", "", ""])[:4])
    image, site, maintenance = classify(commits)

    if args.commits:
        out += ["## ✨ What's new", ""]
        if image:
            out += [commit_line(*c) for c in image]
        else:
            out.append("No change to Celenn OS itself this week: "
                       "only the Fedora updates below.")
        out.append("")
        if site:
            out += ["## 🌐 Website and documentation", ""]
            out += [commit_line(*c) for c in site] + [""]

    out += ["## 📦 Main packages", ""]
    out += main_table(old, new, args.nvidia_old, args.nvidia) + [""]

    if old:
        rows = change_groups(old, new)
        counts = {s: sum(1 for r in rows if r[0] == s) for s in STATUS}
        summary = ", ".join(f"{n} {s}" for s, n in counts.items() if n) or "no change"
        out += ["<details>", f"<summary><b>All package changes</b> ({summary})</summary>", ""]
        if rows:
            out += ["| | Package | Previous | New |", "| :-: | --- | --- | --- |"]
            for status, label, count, before, after in rows:
                name = f"{label} <sub>×{count}</sub>" if count > 1 else label
                out.append(f"| {STATUS[status]} | {name} | {before or ''} | {after or ''} |")
        else:
            out.append("No package changed.")
        out += ["", "</details>", ""]
    else:
        out += [f"First stable release with release notes: {len(new)} packages, "
                "no previous version to compare with.", ""]

    if maintenance:
        out += ["<details>", f"<summary><b>Build and tests</b> ({len(maintenance)})</summary>", ""]
        out += [commit_line(*c) for c in maintenance]
        out += ["", "</details>", ""]

    out += ["## ⬆️ Get it", "",
            "- **Already installed:** nothing to do, or `ujust update` to get it now."]
    if args.site:
        out.append(f"- **New install:** download the live ISO from the [website]({args.site}).")
    if args.image:
        repo_image = args.image.rsplit(":", 1)[0]
        out += ["- **From another bootc system** (Bluefin, Aurora, Bazzite, Fedora Atomic…):", "",
                "  ```bash",
                f"  sudo bootc switch {repo_image}:stable",
                "  sudo systemctl reboot",
                "  ```", ""]
        if args.site:
            out += [f"  NVIDIA graphics cards: `{repo_image}-nvidia:stable`, see the "
                    f"[install guide]({args.site.rstrip('/')}/#install) for Secure Boot.", ""]
        out += [f"This exact build: `{args.image}`", ""]
    links = []
    if args.compare_url:
        links.append(f"[Full diff]({args.compare_url})")
    if args.site:
        links.append(f"[Documentation]({args.site.rstrip('/')}/docs/)")
    if links:
        out += [" · ".join(links), ""]

    text = "\n".join(out)
    if len(text) > MAX_LENGTH:
        cut = text.rfind("\n", 0, MAX_LENGTH)
        text = text[:cut] + "\n\n*(list cut: too many changes for a release page)*\n"
    sys.stdout.write(text)


if __name__ == "__main__":
    main()
