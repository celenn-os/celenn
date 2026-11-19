# Contributing to cosmic-blue

Thanks for helping. cosmic-blue is a small independent project: the image is
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

The texts cosmic-blue adds to the system are in English, French, German,
Spanish, Italian and Brazilian Portuguese. Any change to one of them goes in
every language. To add a language:

- Tours (first steps and live ISO welcome): copy
  `build_files/first_steps/po/cosmic-blue-tour.pot` to `po/<language>.po`
  (for example `nl.po`, or `pt_BR.po` for a regional variant), translate
  every `msgstr`, and add the file to `LANGUAGES` in
  `build_files/first_steps/src/main.rs`.
- Terminal tips: copy `system_files/usr/share/cosmic-blue/tips/en.txt` to
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

The image build fails if a tour text or tip is missing in a language. The
website build fails if a translated page lacks a section, a command, a
screenshot or a language link of the English one.

Proposing new COSMIC defaults: see [docs/cosmic-defaults.md](docs/cosmic-defaults.md).
