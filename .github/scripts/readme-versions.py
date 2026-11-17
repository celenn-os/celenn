#!/usr/bin/env python3
"""Replace the versions block of README.md (between the versions:start and
versions:end comments) with the content of a file.

Usage: readme-versions.py BLOCK_FILE README.md
Exits with an error if README.md has no versions block.
"""
import re
import sys

START = "<!-- versions:start"
END = "<!-- versions:end -->"


def main():
    block_path, readme_path = sys.argv[1:3]
    with open(block_path, encoding="utf-8") as f:
        block = f.read().strip("\n")
    with open(readme_path, encoding="utf-8") as f:
        readme = f.read()
    pattern = re.compile(r"(" + re.escape(START) + r"[^\n]*\n).*?(" + re.escape(END) + r")", re.S)
    if not pattern.search(readme):
        sys.exit(f"No versions block in {readme_path}")
    readme = pattern.sub(lambda m: m.group(1) + block + "\n" + m.group(2), readme, count=1)
    with open(readme_path, "w", encoding="utf-8") as f:
        f.write(readme)


if __name__ == "__main__":
    main()
