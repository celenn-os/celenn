# Bibata Modern sources

Copied from [Bibata_Cursor](https://github.com/ful1e5/Bibata_Cursor) v2.0.7
(commit 35ccfe2), the final release of that project, by Abdulkaiz Khatri,
under the GPL-3.0 (see LICENSE):

- `svg/modern/`: the Bibata Modern drawings, with placeholder colors (shared
  drawings copied in place of the upstream symbolic links)
- `configs/normal/x.build.toml`: sizes, hotspots, frame delays and alias names

`../bibata.py` recolors them with the Celenn OS colors and builds the cursor
theme. Keeping them here means the build no longer depends on an upstream that
publishes no new versions. Renovate still watches it and opens a pull request
on a new release: copy the new sources here as described in `../bibata.sh`.
