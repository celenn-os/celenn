#!/usr/bin/env python3
"""Replace the versions block of README.md (between the versions:start and
versions:end comments) with the content of a file.

With VERSION, the website screenshots in README.md get ?v=VERSION in their
address, so GitHub's image cache shows the screenshots of the new stable.

Usage: readme-versions.py BLOCK_FILE README.md [VERSION]
Exits with an error if README.md has no versions block.
"""
import re
import sys

START = "<!-- versions:start"
END = "<!-- versions:end -->"
SHOT = re.compile(r"(https://celenn-os\.github\.io/celenn/img/screenshots/[a-z0-9-]+\.webp)(\?v=[A-Za-z0-9._-]*)?")


def main():
    block_path, readme_path = sys.argv[1:3]
    version = sys.argv[3] if len(sys.argv) > 3 else ""
    with open(block_path, encoding="utf-8") as f:
        block = f.read().strip("\n")
    with open(readme_path, encoding="utf-8") as f:
        readme = f.read()
    pattern = re.compile(r"(" + re.escape(START) + r"[^\n]*\n).*?(" + re.escape(END) + r")", re.S)
    if not pattern.search(readme):
        sys.exit(f"No versions block in {readme_path}")
    readme = pattern.sub(lambda m: m.group(1) + block + "\n" + m.group(2), readme, count=1)
    if version:
        readme = SHOT.sub(lambda m: m.group(1) + "?v=" + version, readme)
    with open(readme_path, "w", encoding="utf-8") as f:
        f.write(readme)


if __name__ == "__main__":
    main()
