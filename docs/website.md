# Website

The site is static and lives in `site/`. `python3 site/build.py` writes `site/dist` (English at the root, Dutch under `/nl/`). It needs only Python 3.

- `site/build.py`: pages and texts (the `T` dictionary holds all words per language).
- `site/svgs.py`: schematic drawings for the install guide, one set per language.
- `site/src/assets`: `style.css` and `app.js` (hero waterfall, typed demo, morse toy, latest release lookup).
- `site/src/img`: screenshots of the app with numbered badges.
- The changelog page is generated from `CHANGELOG.md`.

## Publish once

On GitHub: Settings, Pages, Source = GitHub Actions. After that every push to `main` that touches `site/` or `CHANGELOG.md`, and every published release, rebuilds the site at https://on3vz.github.io/CWLD_DECODE/.

## Download button

The browser asks the GitHub API for the latest release and fills in version, file, size, date and SHA-256 (when GitHub provides the digest). If that fails, the button points to the release page.

## Add a language

Copy the `nl` block in `T` and `install_body`/`help_body` texts, add the language to the loops in `main()` and `layout()`.
