#!/usr/bin/env python3
"""Generate the cosmic-blue icons drawn on top of Tela circle.

- places: folders (a planet-like disk in the logo blues, with a folder tab),
  the home, desktop, network and trash icons
- mimetypes: common file types (a light page whose symbol and folded corner
  take the color of the type)
- apps: the COSMIC dock buttons (launcher, workspaces, applications) and
  the COSMIC Store and Settings in the Tela circle style; the live ISO's
  welcome tour and installer, and the installed system's first steps tour
- actions: the symbolic icons of the session actions in the panel's power
  menu (lock, log out, suspend, restart); Tela's and COSMIC's are mixed and
  hard to read. The shutdown icon stays Tela's, and these follow its style.

The icon theme (build_files/icon_theme/compose.py) puts them in front of
Tela; every other icon still comes from Tela.

Usage: python3 build.py  (rewrites build_files/icon_theme/layer/)
"""
import os
import shutil
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
LAYER = REPO / "build_files" / "icon_theme" / "layer"
APPS = REPO / "iso_files" / "welcome"
FIRST_STEPS_ICON = REPO / "system_files/usr/share/icons/hicolor/scalable/apps/cosmic-blue-welcome.svg"

# Logo blues (branding/logo/build.py) and the accent (#4f8dff dark, #1d47c4 light)
ORBIT = ("#7fe0ff", "#2f6bff")
MOON = "#e6f3ff"


def stroke(d, width=2.2):
    return (f'<path d="{d}" fill="none" stroke="currentColor" stroke-width="{width}" '
            'stroke-linecap="round" stroke-linejoin="round"/>')


def fill(d):
    return f'<path d="{d}" fill="currentColor"/>'


def dot(x, y, r):
    return fill(f"M{x - r} {y}a{r} {r} 0 1 0 {2 * r} 0a{r} {r} 0 1 0 {-2 * r} 0")


# Symbols drawn in a 24x24 box centered on 0,0, in currentColor
GLYPHS = {
    "documents": stroke("M-6 -9h8l4 4v14h-12z") + stroke("M-3 -1h6M-3 3h6M-3 7h4", 1.8),
    "download": stroke("M0 -9v13M-6 -2l6 6 6-6") + stroke("M-8 9h16"),
    "music": stroke("M-3 6v-13l10 -3v13") + dot(-3, 6, 3.6) + dot(7, 3, 3.6),
    "pictures": stroke("M-9 -7h18v14h-18z") + stroke("M-9 5l6-6 5 5 3-3 4 4", 2) + dot(4, -3.5, 2.2),
    "videos": stroke("M-9 -7h18v14h-18z") + fill("M-2.5 -4v8l7 -4z"),
    "desktop": stroke("M-10 -8h20v13h-20z") + stroke("M-4 9h8M0 5v4"),
    "home": stroke("M-9 0l9-8 9 8M-6 -2v10h12v-10") + fill("M-2 8v-5h4v5z"),
    "templates": stroke("M-8 -9h16v18h-16z") + stroke("M-4 -4h8M-4 0h8M-4 4h4", 1.8),
    "publicshare": dot(-6, 0, 3) + dot(6, -6, 3) + dot(6, 6, 3) + stroke("M-6 0l12-6M-6 0l12 6", 2),
    "network": stroke("M0 -9a9 9 0 1 0 0.01 0z")
    + stroke("M-9 0h18M0 -9c-4 4-4 14 0 18M0 -9c4 4 4 14 0 18", 1.8),
    "recent": stroke("M0 -9a9 9 0 1 0 0.01 0z") + stroke("M0 -5v5l4 3"),
    "trash": stroke("M-9 -6h18M-3 -6v-3h6v3M-7 -6l1.5 15h11l1.5-15") + stroke("M-2 -1v6M2 -1v6", 1.8),
    "trash-full": stroke("M-9 -4h18M-7 -4l1.5 13h11l1.5-13") + stroke("M-5 -7l3-4 3 3 3-3 3 4", 1.8)
    + stroke("M-2 1v4M2 1v4", 1.8),
    "text": stroke("M-6 -6h12M-6 -2h12M-6 2h12M-6 6h7", 2),
    "document": fill("M-7 -8h14v4h-14z") + stroke("M-6 1h12M-6 5h12M-6 9h8", 1.8),
    "spreadsheet": stroke("M-7 -7h14v15h-14zM-7 -2h14M-7 3h14M-1 -7v15", 1.8),
    "presentation": stroke("M-8 -7h16v11h-16z", 2) + stroke("M0 4v5M-4 9h8", 2)
    + fill("M-4 1v-4h2.5v4zM-0.5 1v-6h2.5v6zM3 1v-2.5h2.5v2.5z"),
    "pdf": stroke("M-7 7c3-1 6-6 7-12c0-3-2-3-2 0c0 3 3 9 9 10c2 0 2-2 0-2c-5 0-11 3-14 6c-1 1 0 2 1 1", 1.9),
    "image": stroke("M-9 7l6-7 4 4 3-3 5 6z", 2) + dot(4, -5, 2.6),
    "audio": stroke("M-3 6v-13l9 -2.5v13") + dot(-3, 6, 3.4) + dot(6, 3.5, 3.4),
    "video": fill("M-5 -8v16l13-8z"),
    "archive": fill("M-3 -11h3v2.5h-3zM0 -8.5h3v2.5h-3zM-3 -6h3v2.5h-3zM0 -3.5h3v2.5h-3z")
    + stroke("M-3.5 1h7v8h-7zM-3.5 5h7", 2),
    "script": stroke("M-7 -5l5 5-5 5M1 6h7", 2.4),
    "code": stroke("M-4 -6l-6 6 6 6M4 -6l6 6-6 6M2 -9l-4 18", 2.2),
    "font": stroke("M-8 9l8-18 8 18M-4.5 2h9", 2.4),
    "disc": stroke("M0 -9a9 9 0 1 0 0.01 0z") + dot(0, 0, 2.5),
    "package": stroke("M0 -9l8 4v10l-8 4-8-4v-10zM-8 -5l8 4 8-4M0 -1v10", 2),
    "unknown": stroke("M-4 -4a4 4 0 1 1 6 3.5c-1.5 1-2 2-2 3.5", 2.4) + dot(0, 8, 1.6),
}


def svg(body, defs=""):
    return ('<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 64 64">'
            f"<defs>{defs}</defs>{body}</svg>\n")


def glyph(name, x, y, scale, color):
    if not name:
        return ""
    return f'<g color="{color}" transform="translate({x} {y}) scale({scale})">{GLYPHS[name]}</g>'


# --- Folders -----------------------------------------------------------------

FOLDER_DEFS = (
    '<clipPath id="disk"><circle cx="32" cy="32" r="28"/></clipPath>'
    '<linearGradient id="front" x1=".15" y1="0" x2=".85" y2="1">'
    '<stop offset="0" stop-color="#78aeff"/><stop offset=".55" stop-color="#4f8dff"/>'
    '<stop offset="1" stop-color="#2f6bff"/></linearGradient>'
)


def folder(name, open_=False):
    """A planet-like folder, the size of Tela circle's icons.

    The folder is a disk: dark inside, the back panel with its tab, the
    lighter front panel, which starts lower when the folder is open.
    """
    front = 22.0 if open_ else 18.6
    disk = (
        '<g clip-path="url(#disk)">'
        '<rect x="4" y="4" width="56" height="56" fill="#15338f"/>'
        '<path d="M4 11.3H29.2L33.1 15.2H60V60H4z" fill="#2f63d8"/>'
        f'<path d="M4 {front}H60V60H4z" fill="url(#front)"/>'
        f'<path d="M4 {front + 0.6}H60" stroke="#d6e8ff" stroke-width="1.2" opacity=".8"/>'
        "</g>"
        + glyph(name, 32, front + (60 - front) * 0.45, 1.1, "#ffffff")
    )
    # Drawn at 56 px, scaled to the 54 px disk of Tela circle's icons
    return svg(f'<g transform="translate(32 32) scale({27 / 28:.4f}) translate(-32 -32)">{disk}</g>', FOLDER_DEFS)


# Tela's file names: the icon theme also takes every Tela alias of them.
PLACES = {
    "default-folder": None,
    "default-folder-open": "open",
    "default-folder-documents": "documents",
    "default-folder-download": "download",
    "default-folder-music": "music",
    "default-folder-pictures": "pictures",
    "default-folder-video": "videos",
    "default-folder-templates": "templates",
    "default-folder-publicshare": "publicshare",
    "default-folder-recent": "recent",
    "default-user-desktop": "desktop",
    "default-user-home": "home",
    "default-network": "network",
    "user-trash": "trash",
    "user-trash-full": "trash-full",
}

# --- File types --------------------------------------------------------------

# Symbol -> (color, Tela names). The first name gets the file, the others are
# links to it. Tela's own aliases of all of them follow (compose.py).
MIMETYPES = {
    "text": ("#4f8dff", ["text-x-generic", "text-markdown", "text-x-readme", "text-richtext"]),
    "document": ("#4f8dff", ["x-office-document", "libreoffice-oasis-text", "application-vnd.ms-word",
                             "application-msword-template",
                             "application-vnd.oasis.opendocument.text-template"]),
    "spreadsheet": ("#3fbf7f", ["x-office-spreadsheet", "libreoffice-oasis-spreadsheet",
                                "application-vnd.ms-excel",
                                "application-vnd.oasis.opendocument.spreadsheet-template"]),
    "presentation": ("#ff8a4c", ["x-office-presentation", "libreoffice-oasis-presentation",
                                 "application-vnd.ms-powerpoint",
                                 "application-vnd.oasis.opendocument.presentation-template"]),
    "pdf": ("#ff5d6c", ["application-pdf"]),
    "image": ("#2ec4c4", ["image-x-generic"]),
    "audio": ("#ff6fb0", ["audio-x-generic"]),
    "video": ("#a873ff", ["video-x-generic"]),
    "archive": ("#f0a83a", ["package-x-generic", "application-x-archive", "application-x-zip",
                            "application-x-tar", "application-x-compressed-tar",
                            "application-x-bzip-compressed-tar", "application-x-lzma-compressed-tar",
                            "application-x-gzip", "application-x-bzip", "application-x-rar"]),
    "script": ("#8ea6d8", ["text-x-script", "application-x-shellscript", "application-x-executable"]),
    "code": ("#5fc8ff", ["text-html", "text-xml", "application-json", "application-x-yaml", "text-css"]),
    "font": ("#8ea6d8", ["font-x-generic"]),
    "disc": ("#4f8dff", ["application-x-cd-image", "application-x-iso9660-image"]),
    "package": ("#4f8dff", ["application-x-rpm", "application-vnd.flatpak.ref", "application-vnd.flatpak",
                            "application-vnd.appimage"]),
    "unknown": ("#8a94ad", ["unknown", "application-octet-stream"]),
}

PAGE = "M14 5H42L54 17V55A4 4 0 0 1 50 59H14A4 4 0 0 1 10 55V9A4 4 0 0 1 14 5z"
PAGE_DEFS = ('<linearGradient id="page" x1="0" y1="0" x2="0" y2="1">'
             '<stop offset="0" stop-color="#f7faff"/><stop offset="1" stop-color="#e3ebfa"/></linearGradient>')


def page(name, color):
    """A light page with a folded corner in the color of the file type."""
    body = (f'<path d="{PAGE}" fill="url(#page)" stroke="#b9c6e2" stroke-width="1"/>'
            f'<path d="M42 5V13A4 4 0 0 0 46 17H54z" fill="{color}"/>'
            + glyph(name, 32, 34, 1.05, color))
    return svg(body, PAGE_DEFS)


# --- Applications ------------------------------------------------------------

APP_DEFS = (
    '<linearGradient id="night" x1="0" y1="0" x2="0" y2="1">'
    '<stop offset="0" stop-color="#1a2d6b"/><stop offset="1" stop-color="#0b1736"/></linearGradient>'
    f'<linearGradient id="orbit" x1="0" y1="1" x2="1" y2="0"><stop offset="0" stop-color="{ORBIT[0]}"/>'
    f'<stop offset="1" stop-color="{ORBIT[1]}"/></linearGradient>'
    '<linearGradient id="planet" x1=".2" y1="1" x2=".8" y2="0"><stop offset="0" stop-color="#cfeeff"/>'
    '<stop offset=".45" stop-color="#4f98f5"/><stop offset="1" stop-color="#14399e"/></linearGradient>'
)


def app(center):
    """The logo mark on a night-blue disk, with `center` inside the orbit."""
    return svg(
        '<circle cx="32" cy="32" r="28" fill="url(#night)"/>'
        '<circle cx="32" cy="32" r="27.4" fill="none" stroke="#4f8dff" stroke-opacity=".35" stroke-width="1.2"/>'
        '<g transform="translate(31 32) scale(.5)">'
        '<path d="M30.3 -25.4A40 40 0 1 0 30.3 25.4" fill="none" stroke="url(#orbit)" '
        'stroke-width="13" stroke-linecap="round"/>'
        f'<circle cx="38" cy="0" r="6.5" fill="{MOON}"/></g>' + center,
        APP_DEFS,
    )


# --- COSMIC dock buttons ------------------------------------------------------

# Drawn like Tela circle's application icons: a disk in the accent, white
# symbols, the less important parts in translucent white.
TELA_SHADE = '<path d="M5 32a27 27 0 0 0 54 0z" opacity=".1"/>'
STORE_COLOR = "#2ec4c4"
SETTINGS_COLOR = "#5f6b86"


def tela_app(body, color="#4f8dff"):
    return svg(f'<circle cx="32" cy="32" r="27" fill="{color}"/>' + TELA_SHADE + body)


def tile(x, y, size, opacity=None, radius=1.6):
    alpha = f' opacity="{opacity}"' if opacity else ""
    return f'<rect x="{x}" y="{y}" width="{size}" height="{size}" rx="{radius}" fill="#fff"{alpha}/>'


# The app library: a grid of applications
APPLICATIONS = tela_app("".join(
    tile(18 + col * 10, 18 + row * 10, 8, None if (row, col) == (1, 1) else ".75")
    for row in range(3) for col in range(3)
))
# Workspaces: two windows, one behind the other
WORKSPACES = tela_app(
    '<rect x="15" y="17" width="23" height="17" rx="2" fill="#fff" opacity=".45"/>'
    '<rect x="26" y="28" width="23" height="19" rx="2" fill="#fff"/>'
    '<path d="M28 28h19a2 2 0 0 1 2 2v2H26v-2a2 2 0 0 1 2-2z" fill="#2f6bff"/>'
)
# The launcher: search over applications
LAUNCHER = tela_app(
    tile(16, 16, 9, ".45") + tile(16, 27, 9, ".45") + tile(27, 16, 9, ".45")
    + '<circle cx="34" cy="34" r="9.5" fill="#4f8dff"/>'
    '<circle cx="34" cy="34" r="7.5" fill="none" stroke="#fff" stroke-width="3.6"/>'
    '<path d="M39.6 39.6l7.4 7.4" stroke="#fff" stroke-width="4.4" stroke-linecap="round"/>'
)

# The store: a shopping bag with the logo's "c"
STORE = tela_app(
    '<path d="M25 24v-3a7 7 0 0 1 14 0v3" fill="none" stroke="#fff" stroke-width="3" stroke-linecap="round"/>'
    '<path d="M18.5 24h27a1.5 1.5 0 0 1 1.5 1.6l-1.4 19.6a3 3 0 0 1-3 2.8H21.4a3 3 0 0 1-3-2.8L17 25.6'
    'a1.5 1.5 0 0 1 1.5-1.6z" fill="#fff"/>'
    f'<path d="M37.2 31.5A6.5 6.5 0 1 0 37.2 40.5" fill="none" stroke="{STORE_COLOR}" stroke-width="2.6" '
    'stroke-linecap="round"/>'
    f'<circle cx="32" cy="36" r="2.4" fill="{STORE_COLOR}"/>',
    STORE_COLOR,
)
# The settings: two sliders
SETTINGS = tela_app(
    '<rect x="16" y="21" width="32" height="4" rx="2" fill="#fff" opacity=".45"/>'
    '<rect x="16" y="39" width="32" height="4" rx="2" fill="#fff" opacity=".45"/>'
    '<rect x="16" y="21" width="22" height="4" rx="2" fill="#fff"/>'
    '<rect x="16" y="39" width="10" height="4" rx="2" fill="#fff"/>'
    '<circle cx="38" cy="23" r="5" fill="#fff"/><circle cx="26" cy="41" r="5" fill="#fff"/>',
    SETTINGS_COLOR,
)

# Icon -> names (the desktop entries of the dock buttons and of the apps
# they open). The first name gets the file, the others are links to it.
DOCK = {
    "applications": (APPLICATIONS, ["com.system76.CosmicPanelAppButton", "com.system76.CosmicAppLibrary"]),
    "workspaces": (WORKSPACES, ["com.system76.CosmicPanelWorkspacesButton", "com.system76.CosmicWorkspaces",
                                "com.system76.CosmicAppletWorkspaces"]),
    "launcher": (LAUNCHER, ["com.system76.CosmicPanelLauncherButton", "com.system76.CosmicLauncher"]),
    "store": (STORE, ["com.system76.CosmicStore"]),
    "settings": (SETTINGS, ["com.system76.CosmicSettings"]),
}

# Session action symbolic icons, 16x16, in Tela's symbolic gray. They are
# drawn with strokes, then outlined into plain fills: GTK recolors symbolic
# icons by forcing their fill, which would fill an unoutlined stroke.
# (names, path); the first name gets the file, the others are links to it.
SYMBOLIC_COLOR = "#565656"
ACTIONS = [
    # A crescent moon
    (["system-suspend-symbolic"],
     "M7.31,2.04 C6.12,4.03 6.45,6.57 8.11,8.18 C9.77,9.8 12.32,10.06 14.28,8.82 C14,12.13 11.2,14.66 7.88,14.6 C4.56,14.54 1.86,11.91 1.71,8.59 C1.55,5.27 4,2.4 7.31,2.04 Z"),
    # An open ring with a clockwise arrowhead, like the shutdown ring
    (["system-reboot-symbolic", "system-restart-symbolic"],
     "M14.07,6.17 Q16.18,10.68 12.46,14 Q8.74,17.31 4.5,14.71 Q0.25,12.11 1.52,7.29 Q2.65,3.02 6.69,2.4 L6.63,0.84 Q6.63,0.71 6.7,0.6 Q6.8,0.47 6.96,0.44 Q7.12,0.41 7.26,0.5 L10.74,2.88 Q10.8,2.91 10.83,2.96 Q10.94,3.09 10.92,3.26 Q10.9,3.42 10.77,3.52 L7.46,6.14 Q7.35,6.22 7.22,6.23 Q7.06,6.23 6.94,6.12 Q6.82,6.01 6.81,5.84 L6.76,4.43 Q4.2,4.98 3.45,7.8 Q2.57,11.18 5.54,13.01 Q8.52,14.83 11.13,12.51 Q13.74,10.18 12.26,7.01 Q12.09,6.64 12.23,6.25 Q12.37,5.86 12.74,5.68 Q13.12,5.51 13.51,5.65 Q13.9,5.79 14.07,6.17 Z"),
    # A door frame with an arrow going out
    (["system-log-out-symbolic"],
     "M8,3.5 L4.5,3.5 L4.5,12.5 L8,12.5 Q8.41,12.5 8.71,12.79 Q9,13.09 9,13.5 Q9,13.91 8.71,14.21 Q8.41,14.5 8,14.5 L3.5,14.5 Q3.09,14.5 2.79,14.21 Q2.5,13.91 2.5,13.5 L2.5,2.5 Q2.5,2.09 2.79,1.79 Q3.09,1.5 3.5,1.5 L8,1.5 Q8.41,1.5 8.71,1.79 Q9,2.09 9,2.5 Q9,2.91 8.71,3.21 Q8.41,3.5 8,3.5 Z M7,7 L11.59,7 L10.29,5.71 Q10,5.41 10,5 Q10,4.59 10.29,4.29 Q10.59,4 11,4 Q11.41,4 11.71,4.29 L14.71,7.29 Q15,7.59 15,8 Q15,8.41 14.71,8.71 L11.71,11.71 Q11.41,12 11,12 Q10.59,12 10.29,11.71 Q10,11.41 10,11 Q10,10.59 10.29,10.29 L11.59,9 L7,9 Q6.59,9 6.29,8.71 Q6,8.41 6,8 Q6,7.59 6.29,7.29 Q6.59,7 7,7 Z"),
    # A closed padlock with a keyhole
    (["system-lock-screen-symbolic"],
     "M12,7.09 C12.58,7.29 13,7.85 13,8.5 L13,13.5 C13,14.33 12.33,15 11.5,15 L4.5,15 C3.67,15 3,14.33 3,13.5 L3,8.5 C3,7.85 3.42,7.29 4,7.09 L4,5.5 Q4,1.5 8,1.5 Q12,1.5 12,5.5 L12,7.09 Z M6,7 L10,7 L10,5.5 Q10,3.5 8,3.5 Q6,3.5 6,5.5 L6,7 Z M8,9.2 C7.46,9.2 6.98,9.56 6.84,10.09 C6.7,10.61 6.93,11.17 7.4,11.44 L7.4,13 L8.6,13 L8.6,11.44 C9.07,11.17 9.3,10.61 9.16,10.09 C9.02,9.56 8.54,9.2 8,9.2 Z"),
]


def symbolic(d):
    return ('<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16">'
            f'<path fill="{SYMBOLIC_COLOR}" fill-rule="evenodd" d="{d}"/></svg>\n')


def main():
    shutil.rmtree(LAYER, ignore_errors=True)
    places = LAYER / "places"
    mimetypes = LAYER / "mimetypes"
    apps = LAYER / "apps"
    places.mkdir(parents=True)
    mimetypes.mkdir()
    apps.mkdir()
    actions = LAYER / "actions"
    actions.mkdir()

    for names, d in ACTIONS:
        (actions / f"{names[0]}.svg").write_text(symbolic(d))
        for alias in names[1:]:
            os.symlink(f"{names[0]}.svg", actions / f"{alias}.svg")

    for icon, names in DOCK.values():
        (apps / f"{names[0]}.svg").write_text(icon)
        for alias in names[1:]:
            os.symlink(f"{names[0]}.svg", apps / f"{alias}.svg")

    for name, symbol in PLACES.items():
        is_open = symbol == "open"
        (places / f"{name}.svg").write_text(folder(None if is_open else symbol, open_=is_open))

    for symbol, (color, names) in MIMETYPES.items():
        (mimetypes / f"{names[0]}.svg").write_text(page(symbol, color))
        for alias in names[1:]:
            os.symlink(f"{names[0]}.svg", mimetypes / f"{alias}.svg")

    welcome = app('<circle cx="30" cy="32" r="6.5" fill="url(#planet)"/>')
    (APPS / "cosmic-blue-welcome.svg").write_text(welcome)
    # Also the icon of the installed system's first steps tour
    (FIRST_STEPS_ICON).write_text(welcome)
    (APPS / "cosmic-blue-installer.svg").write_text(app(glyph("download", 30, 32, 0.85, "#ffffff")))


if __name__ == "__main__":
    main()
