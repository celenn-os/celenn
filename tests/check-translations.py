#!/usr/bin/python3
"""Every text the apps translate in place, tr("..."), is in the template
(.pot) of each program that compiles it, so that it cannot slip through
untranslated. The programs' own `--check` (run by the image build) then
makes sure every template text is translated in every language; it only
sees the lists of texts the programs keep (TEXTS), which this check
completes. A text is also defined only once in each template and
translation: gettext's own tools (msgfmt, translation editors) refuse a
file with a text twice. Run by tests/lint.sh."""

import pathlib
import re
import sys

CRATE = pathlib.Path(__file__).resolve().parent.parent / "build_files/first_steps"
# Each program: its source root and its template
PROGRAMS = {
    "src/main.rs": "po/cosmic-blue-tour.pot",
    "src/tools.rs": "po/tools/cosmic-blue-tools.pot",
    "src/parental.rs": "po/parental/cosmic-blue-parental-controls.pot",
    "src/parental_daemon.rs": "po/parental/cosmic-blue-parental-controls.pot",
}
TR = re.compile(r'\btr\(\s*"((?:[^"\\]|\\.)*)"\s*\)', re.S)
MOD = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;", re.M)
MSGID = re.compile(r'^msgid "((?:[^"\\]|\\.)*)"\n((?:"(?:[^"\\]|\\.)*"\n)*)', re.M)


def unescape(text):
    return re.sub(r"\\(.)", lambda m: {"n": "\n", "t": "\t"}.get(m.group(1), m.group(1)), text)


def rust_unescape(text):
    # A backslash at the end of a line joins it to the next, without the
    # next line's indentation
    text = re.sub(r"\\\n\s*", "", text)
    return unescape(text)


def msgids(path):
    ids = []
    for match in MSGID.finditer(path.read_text()):
        more = re.findall(r'"((?:[^"\\]|\\.)*)"', match.group(2))
        ids.append(unescape(match.group(1) + "".join(more)))
    return ids


def template(path):
    return set(msgids(path))


errors = 0
for po in sorted(CRATE.glob("po/**/*.po*")):
    seen = set()
    for text in msgids(po):
        if text and text in seen:
            print(f"{po.relative_to(CRATE)}: defined twice: {text!r}")
            errors += 1
        seen.add(text)
for root, pot in PROGRAMS.items():
    known = template(CRATE / pot)
    files = [CRATE / root]
    files += [CRATE / "src" / f"{name}.rs" for name in MOD.findall(files[0].read_text())]
    for file in files:
        source = file.read_text()
        # Not the unit tests
        source = source.split("#[cfg(test)]")[0]
        for match in TR.finditer(source):
            text = rust_unescape(match.group(1))
            if text not in known:
                line = source.count("\n", 0, match.start()) + 1
                print(f"{file.relative_to(CRATE)}:{line}: not in {pot}: {text!r}")
                errors += 1
print(f"translations: {errors} problem(s) found")
sys.exit(1 if errors else 0)
