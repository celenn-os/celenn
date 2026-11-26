#!/usr/bin/env python3
"""Check that every app cosmic-blue offers has an icon in its icon theme.

The apps are the Flatpaks of the first steps tour and Tools lists
(build_files/first_steps/src: flatpak("Name", "id") and Flatpak("id")) and
the apps installed on first boot (flatpak preinstall file, not its
runtimes). A Flatpak brings its own icon, but in another style: in the App
Library and the app lists it would stand out among the round Tela icons. An app is covered when the theme
has an icon named after its id, or after one of its launchers (id.writer,
id-writer, as LibreOffice's).

A missing icon gets one in the cosmic-blue layer (OFFERED in
branding/icons/build.py).

The theme directories are the cosmic-blue theme and the Tela themes it
inherits its application icons from (build_files/icon_theme/compose.py), not
the fallbacks (Adwaita, hicolor), whose icons are in another style.

Usage: check_app_icons.py <first_steps src dir> <preinstall file> <theme dir>...
"""

import configparser
import os
import re
import sys

CONST = re.compile(r'const\s+(\w+)\s*:\s*&str\s*=\s*"([^"]+)"')
LIST = re.compile(r'\bflatpak\(\s*"[^"]*"\s*,\s*("[^"]+"|\w+)\s*,?\s*\)')
KIND = re.compile(r'\bFlatpak\(\s*("[^"]+"|[\w:]+)\s*\)')


def offered(src, preinstall):
    sources = {}
    for name in sorted(os.listdir(src)):
        if name.endswith(".rs"):
            with open(os.path.join(src, name), encoding="utf-8") as f:
                sources[name] = f.read()
    consts = {}
    for text in sources.values():
        consts.update(CONST.findall(text))
    apps = set()
    for text in sources.values():
        for match in [*LIST.findall(text), *KIND.findall(text)]:
            if match.startswith('"'):
                apps.add(match.strip('"'))
            elif match.split("::")[-1] in consts:
                apps.add(consts[match.split("::")[-1]])
            # Anything else is a variable (Flatpak(app) in a match arm)
    parser = configparser.ConfigParser(interpolation=None)
    parser.read(preinstall, encoding="utf-8")
    for section in parser.sections():
        # Apps only: runtimes (GTK themes...) have no icon
        app = section.removeprefix("Flatpak Preinstall ")
        if app != section and not parser.getboolean(section, "IsRuntime", fallback=False):
            apps.add(app)
    return apps


def app_icons(themes):
    names = set()
    for root, _dirs, files in (entry for theme in themes for entry in os.walk(theme, followlinks=True)):
        if os.sep + "apps" in root + os.sep and "symbolic" not in root:
            names.update(os.path.splitext(f)[0] for f in files)
    return names


def main():
    if len(sys.argv) < 4:
        sys.exit(__doc__)
    src, preinstall, *themes = sys.argv[1:]
    apps = offered(src, preinstall)
    icons = app_icons(themes)
    if len(apps) < 30 or len(icons) < 1000:
        sys.exit(f"Found only {len(apps)} offered apps and {len(icons)} app icons: check the paths")
    missing = sorted(
        app for app in apps
        if app not in icons and not any(icon.startswith((app + ".", app + "-")) for icon in icons)
    )
    print(f"{len(apps)} offered apps, {len(apps) - len(missing)} with an icon in the theme")
    if missing:
        sys.exit("No icon in the theme for: " + ", ".join(missing)
                 + " (add one to OFFERED in branding/icons/build.py)")


if __name__ == "__main__":
    main()
