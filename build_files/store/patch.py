#!/usr/bin/python3
"""Patches COSMIC Store for cosmic-blue: FlatPark as a recommended source.

COSMIC Store's "Software repositories" panel lists two built-in recommended
Flatpak sources (Flathub, COSMIC Flatpak), each with an Add or Remove button,
for the user's own Flatpak installation. cosmic-blue offers FlatPark but
leaves it off (its apps are not reviewed like Flathub's), so it becomes a
third recommended source, off until the user adds it:

1. The FlatPark entry reads its descriptor from the image
   (/usr/share/cosmic-blue/flatpak/flatpark.flatpakrepo, from
   projectbluefin/common), so it follows common's updates. Its Filter= key
   hides every app that is also on Flathub. It requires Flathub: adding it
   adds Flathub if missing, removing Flathub removes it too. No entry if the
   file is missing.
2. FlatPark is added with priority 0, below Flathub and COSMIC (1).

`ujust toggle-flatpark` turns on or off the same remote.
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
    "src/main.rs",
    """                //TODO: can this be defined in flatpakrepo file?
                requires: vec!["flathub".to_string()],
            });
""",
    """                //TODO: can this be defined in flatpakrepo file?
                requires: vec!["flathub".to_string()],
            });
            // cosmic-blue: FlatPark, offered but off by default
            static FLATPARK: std::sync::OnceLock<Option<Vec<u8>>> = std::sync::OnceLock::new();
            if let Some(data) = FLATPARK.get_or_init(|| {
                std::fs::read("/usr/share/cosmic-blue/flatpak/flatpark.flatpakrepo").ok()
            }) {
                sources.push(Source {
                    backend_name: BackendName::FlatpakUser,
                    id: "flatpark".to_string(),
                    name: "FlatPark".to_string(),
                    kind: SourceKind::Recommended {
                        data: data.as_slice(),
                        enabled: false,
                    },
                    requires: vec!["flathub".to_string()],
                });
            }
""",
)

replace(
    "src/backend/flatpak.rs",
    """                for add in adds.iter() {
                    remotes.push(Remote::from_file(&add.id, &glib::Bytes::from(&add.data))?);
                }
""",
    """                for add in adds.iter() {
                    let remote = Remote::from_file(&add.id, &glib::Bytes::from(&add.data))?;
                    // cosmic-blue: FlatPark comes after every other source
                    if add.id == "flatpark" {
                        remote.set_prio(0);
                    }
                    remotes.push(remote);
                }
""",
)
