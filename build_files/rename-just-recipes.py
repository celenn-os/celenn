#!/usr/bin/env python3
"""Rename, regroup or hide recipes in a set of justfiles.

Usage: rename-just-recipes.py FILE... -- OP...

Each OP is one of:
  old=new         rename recipe `old` to `new`
  old=new@Group   rename it and put it in [group('Group')]
  name@Group      put it in [group('Group')]
  -name           hide it from `ujust` ([private]); it still runs
  old=new>public  rename it, but make `ujust old` calls use `public`
                  (for a hidden _name that a public command now wraps)
  ...|Text        (after any of the above) new description, shown by `ujust`

Used on projectbluefin/common's ujust files so that every listed command
reads verb-object, like cosmic-blue's own (install-, toggle-, reset-,
check-, show-...). A recipe starts at an unindented "name ...:" line; the
[attribute] and comment lines right above it belong to it. Calls of a renamed
recipe in any of the files (`ujust old`, `just --justfile ... old`) follow
the rename. 60-custom.just keeps the old names as hidden aliases. Fails if a
recipe is not found, so that an upstream rename is noticed at build time.
"""
import re
import sys

HEADER = re.compile(r"^(@?)([A-Za-z_][\w-]*)(\b(?!\s*:=).*:(?!=).*)$")
SKIP = ("alias ", "set ", "import", "mod ", "export ")


def parse_ops(ops):
    plan = {}
    for op in ops:
        private = op.startswith("-")
        op = op.lstrip("-")
        op, _, doc = op.partition("|")
        group = None
        if "@" in op:
            op, group = op.split("@", 1)
        old, _, new = op.partition("=")
        new, _, public = new.partition(">")
        plan[old] = {"new": new or old, "public": public or new or old,
                     "group": group, "private": private, "doc": doc}
    return plan


def rewrite(lines, plan, found):
    out, pending = [], []
    for line in lines:
        if line.startswith(("#", "[")):
            pending.append(line)
            continue
        header = HEADER.match(line.rstrip("\n"))
        if header and not line.startswith(SKIP) and header.group(2) in plan:
            name = header.group(2)
            spec = plan[name]
            found.add(name)
            attrs = [a for a in pending if a.startswith("[")]
            comments = [c for c in pending if not c.startswith("[")]
            if spec["doc"]:
                comments = comments[:-1] + [f"# {spec['doc']}\n"]
            if spec["group"]:
                attrs = [a for a in attrs if not a.startswith("[group(")]
                attrs.append(f"[group('{spec['group']}')]\n")
            if spec["private"] and "[private]\n" not in attrs:
                attrs.append("[private]\n")
            out.extend(comments + attrs)
            out.append(f"{header.group(1)}{spec['new']}{header.group(3)}\n")
            pending = []
            continue
        out.extend(pending)
        out.append(line)
        pending = []
    out.extend(pending)
    return out


def rewrite_calls(text, plan):
    for old, spec in plan.items():
        if spec["new"] == old:
            continue
        text = re.sub(
            r'(\bjust --justfile (?:"[^"]*"|\S+)\s+)' + re.escape(old) + r"(?![\w-])",
            lambda m: m.group(1) + spec["new"],
            text,
        )
        text = re.sub(
            r"(\bujust\s+)" + re.escape(old) + r"(?![\w-])",
            lambda m: m.group(1) + spec["public"],
            text,
        )
    return text


def main() -> int:
    split = sys.argv.index("--")
    files, plan = sys.argv[1:split], parse_ops(sys.argv[split + 1:])
    found = set()
    for path in files:
        with open(path, encoding="utf-8") as f:
            lines = f.read().splitlines(keepends=True)
        text = "".join(rewrite(lines, plan, found))
        with open(path, "w", encoding="utf-8") as f:
            f.write(rewrite_calls(text, plan))
    missing = set(plan) - found
    if missing:
        print(f"recipes not found: {' '.join(sorted(missing))}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
