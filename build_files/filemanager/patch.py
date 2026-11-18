#!/usr/bin/python3
"""Patches COSMIC Files for cosmic-blue: special folder icons under /var/home.

COSMIC Files gives the XDG user folders (Documents, Downloads, Music, ...)
their own icons (folder-documents, folder-download, ...) by looking up each
folder's path in a table built from the user's folders ($HOME/Documents...).
But it opens every location by its canonical path, and on cosmic-blue, as on
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


def replace(rel, old, new):
    path = src / rel
    text = path.read_text()
    count = text.count(old)
    if count != 1:
        sys.exit(f"{rel}: expected 1 match, found {count}:\n{old}")
    path.write_text(text.replace(old, new))


replace(
    "src/tab.rs",
    """        special_dirs.insert(dir, "user-home");
    }
    special_dirs
});
""",
    """        special_dirs.insert(dir, "user-home");
    }
    // cosmic-blue: locations are opened by their canonical path (/home is a
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
