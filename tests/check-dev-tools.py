#!/usr/bin/python3
"""`ujust install-dev-tools` (60-custom.just) offers the same developer tools,
in the same groups and order, as the list of the tour and Tools
(build_files/first_steps/src/devtools.rs), installed from the same place,
with the same ones ticked first (apps.rs, Development's `recommended`). The
terminal menu adds Lima at the end. Run by tests/lint.sh."""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
RUST = ROOT / "build_files/first_steps/src"
JUST = ROOT / "system_files/usr/share/ublue-os/just/60-custom.just"
TERMINAL_ONLY = [["Virtual machine", ("Lima", "ujust", "lima")]]


def rust_groups():
    source = (RUST / "devtools.rs").read_text()
    body = source[source.index("pub const DEV_TOOLS") :]
    body = body[: body.index("\n];")]
    groups = []
    for heading, tools in re.findall(r'\(\s*"([^"]+)",\s*&\[(.*?)\n        \],\s*\)', body, re.S):
        group = [heading]
        for name, kind in re.findall(r'name: "([^"]+)",\s*kind: ToolKind::(.*?),\n', tools):
            if kind.startswith("Formula"):
                names = " ".join(re.findall(r'"([^"]+)"', kind))
                group.append((name, "formula", names))
            elif kind.startswith("Cask"):
                group.append((name, "cask", re.search(r'"([^"]+)"', kind).group(1)))
            else:
                group.append((name, "flatpak", re.search(r'"([^"]+)"', kind).group(1)))
        groups.append(group)
    return groups


def just_groups():
    recipe = JUST.read_text()
    recipe = recipe[recipe.index("_dev-tools action:") :]
    array = re.search(r"\n    tools=\(\n(.*?)\n    \)", recipe, re.S).group(1)
    groups = []
    for entry in re.findall(r'"([^"]*)"', array):
        if "|" in entry:
            groups[-1].append(tuple(entry.split("|")))
        else:
            groups.append([entry])
    recommended = re.search(r"\n    recommended=\((.*?)\)", recipe).group(1)
    return groups, re.findall(r'"([^"]+)"', recommended)


def rust_recommended():
    apps = (RUST / "apps.rs").read_text()
    development = apps[apps.index('title: "Development"') :]
    return re.findall(r'"([^"]+)"', re.search(r"recommended: &\[(.*?)\]", development).group(1))


expected = rust_groups() + TERMINAL_ONLY
found, recommended = just_groups()
errors = 0
if not expected[0][1:]:
    print("check-dev-tools: could not read DEV_TOOLS in devtools.rs")
    errors += 1
if found != expected:
    print("ujust install-dev-tools and the tour/Tools list differ:")
    print("  devtools.rs (+ Lima):", expected)
    print("  60-custom.just:      ", found)
    errors += 1
if recommended != rust_recommended():
    print(f"Ticked first: 60-custom.just {recommended}, apps.rs {rust_recommended()}")
    errors += 1
print(f"dev tools: {errors} problem(s) found")
sys.exit(1 if errors else 0)
