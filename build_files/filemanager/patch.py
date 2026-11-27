#!/usr/bin/python3
"""Patches COSMIC Files for Celenn OS: special folder icons under /var/home.

COSMIC Files gives the XDG user folders (Documents, Downloads, Music, ...)
their own icons (folder-documents, folder-download, ...) by looking up each
folder's path in a table built from the user's folders ($HOME/Documents...).
But it opens every location by its canonical path, and on Celenn OS, as on
every Fedora Atomic system, $HOME is /home/<user> while /home is a link to
/var/home: the folders are shown as /var/home/<user>/Documents, never found in
the table, and all get the plain folder icon.

The patch also puts the canonical path of each user folder in the table.
Each replacement must match exactly once, so the build fails loudly when a
new upstream version changes this code. Usage: patch.py <source dir>
"""

import pathlib
import sys

src = pathlib.Path(sys.argv[1])


def replace(rels, old, new):
    """Replaces old in the one file of rels (tried in turn) that holds it."""
    found = [rel for rel in rels if (src / rel).exists() and old in (src / rel).read_text()]
    if len(found) != 1:
        sys.exit(f"{', '.join(rels)}: expected 1 file with a match, found {len(found)}:\n{old}")
    path = src / found[0]
    text = path.read_text()
    count = text.count(old)
    if count != 1:
        sys.exit(f"{found[0]}: expected 1 match, found {count}:\n{old}")
    path.write_text(text.replace(old, new))


# The table is in src/tab.rs up to COSMIC 1.11, in src/item/mod.rs since 1.12
replace(
    ["src/tab.rs", "src/item/mod.rs"],
    """        special_dirs.insert(dir, "user-home");
    }
    special_dirs
});
""",
    """        special_dirs.insert(dir, "user-home");
    }
    // Celenn OS: locations are opened by their canonical path (/home is a
    // link to /var/home on Fedora Atomic systems), so find them by it too
    let canonical: Vec<_> = special_dirs
        .iter()
        .filter_map(|(dir, icon)| {
            fs::canonicalize(dir)
                .ok()
                .filter(|path| path != dir)
                .map(|path| (path, *icon))
        })
        .collect();
    special_dirs.extend(canonical);
    special_dirs
});
""",
)
