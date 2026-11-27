# Contributing to Celenn OS

Thanks for helping. Celenn OS is a small independent project: the image is
built, tested and published by CI from this repository.

## Reporting

- Bugs: run `ujust report-bug` on the affected computer, it opens a filled-in
  issue. Bugs that also happen with COSMIC on other systems belong to
  [COSMIC](https://github.com/pop-os/cosmic-epoch/issues).
- Questions and ideas: [Discussions](https://github.com/cosmic-blue-os/cosmic-blue/discussions).
- Security problems: privately, see [SECURITY.md](SECURITY.md).

## Changes

1. Open a pull request against `main`. Everything in the repository is in
   English: code, comments, docs and commit messages.
2. CI builds the image, installs it in a virtual machine, boots it and checks
   the desktop (screenshots in the run summary); shell scripts and workflows
   are linted. A pull request is merged once this is green.
3. Pull requests are squash-merged.

Where things are:

- `Containerfile`, `build_files/`: the image build, one numbered script per
  step; `build_files/99-tests.sh` checks the result.
- `system_files/`: files copied as they are into the image.
- `tests/`: the virtual machine tests run by CI.
- `website/`: the website. Every text change goes in the English page and
  in each translation (`website/de/`, `es/`, `fr/`, `it/`, `pt-br/`).
- `branding/`: the logo, wallpapers and icons, and how to regenerate them.

## Translations

The texts Celenn OS adds to the system are in English, French, German,
Spanish, Italian and Brazilian Portuguese. Any change to one of them goes in
every language. To add a language:

- Apps (`build_files/first_steps`), one template per program: copy it to
  `<language>.po` in the same folder (for example `nl.po`, or `pt_BR.po` for
  a regional variant), translate every `msgstr`, and add the file to
  `LANGUAGES` in the program's source:
  - tours, first steps and live ISO welcome: `po/celenn-tour.pot`,
    `src/main.rs`;
  - Celenn OS Tools (with Updates): `po/tools/celenn-tools.pot`,
    `src/tools.rs`;
  - Parental controls (the app and the login screen's message):
    `po/parental/celenn-parental-controls.pot`, `src/parental.rs` and
    `src/parental_daemon.rs`.
- Password prompts of the apps: add a `<description xml:lang="…">` and a
  `<message xml:lang="…">` to each action of
  `system_files/usr/share/polkit-1/actions/io.github.celenn_os.*.policy`.
- Terminal tips: copy `system_files/usr/share/celenn/tips/en.txt` to
  `<language>.txt`, one tip per line, in the same order.
- Launchers: add `Name[<language>]` and `Comment[<language>]` (and
  `Keywords[<language>]`) to the `.desktop` files of `system_files/` and
  `iso_files/welcome/`.
- COSMIC Store texts: add an `append()` for the language in
  `build_files/store/patch.py`.
- Website: copy the English pages into `website/<language>/` (fixing the
  relative paths, as in `website/fr/`), translate them, add the language to
  `languages` in `website/build.sh`, and add it to the language menu and the
  `hreflang` links of every page. Add its copy button labels to
  `website/site.js`.

A new text in an app goes through `tr("…")` and into its program's
template and every `.po` file. The pull request lint fails if a `tr("…")`
text is missing from its template (`tests/check-translations.py`), and the
image build fails if an app text or a tip is missing in a language. The
website build fails if a translated page lacks a section, a command, a
screenshot or a language link of the English one.

Proposing new COSMIC defaults: see [docs/cosmic-defaults.md](docs/cosmic-defaults.md).
Running the project (stable promotions, COSMIC and Fedora updates, CI
failures): see [docs/maintaining.md](docs/maintaining.md).
