# nemu source registry

The recommended source directory built into [nemu](https://nemu.pm).

```
https://nemu-pm.github.io/source-registry/index.min.json
```

Only works whose rights holder publishes them under a free licence and works in the public domain are listed, plus sources that only reach a server you run. Details for each entry are in [`curation/entries.json`](curation/entries.json).

## Sources

| Source | Languages | Licence | Author |
| --- | --- | --- | --- |
| Pepper&Carrot (`multi.peppercarrot`) | en, ja, zh, fr, de, es, it, pt, ru, ko | [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/), [AI derivation prohibited](https://www.davidrevoy.com/article1178/license-update-ai-derivation-prohibited-on-all-my-art-lore-stories-and-comics) | [David Revoy](https://www.peppercarrot.com) |
| Mini Fantasy Theater (`multi.minifantasytheater`) | en, ja, fr, es, de, ca, nn, sl, tok | [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/), [AI derivation prohibited](https://www.davidrevoy.com/article1178/license-update-ai-derivation-prohibited-on-all-my-art-lore-stories-and-comics) | [David Revoy](https://www.peppercarrot.com/en/webcomics/miniFantasyTheater.html) |
| NDL Public Domain Picks (`ja.ndlpublicdomain`) | ja | [Public domain](https://www.ndl.go.jp/en/use/reproduction/index.html) | Seven works; scans by the [National Diet Library, Japan](https://dl.ndl.go.jp) |
| Little Nemo in Slumberland (`en.littlenemo`) | en | [Public domain](https://commons.wikimedia.org/wiki/Category:Little_Nemo_in_Slumberland) | Winsor McCay; scans from Wikimedia Commons |
| LANraragi (`multi.lanraragi`) | multi | — (your own server) | [LANraragi](https://github.com/Difegue/LANraragi) / [Aidoku Community](https://github.com/Aidoku-Community/sources) |

LANraragi has no content of its own; it connects to a server you run.

Pepper&Carrot reads the site's official [API](https://www.peppercarrot.com/ld/documentation/130_API.html). Translators are credited on each chapter. Mini Fantasy Theater reads the site's series page, which is its only official episode list, and ends each chapter on a credits page taken from the site's credit files.

Both series carry the author's "AI derivation prohibited" notice on top of their Creative Commons licence. The [2026-10-08 statement](https://www.davidrevoy.com/article1178/license-update-ai-derivation-prohibited-on-all-my-art-lore-stories-and-comics) prohibits AI-generated derivatives; nemu doesn't make any. Creative Commons has no term for this, so the published index keeps the SPDX-style licence in `contentLicense` and the notice in `contentLicenseNote`. We'll remove either entry if the author asks.

NDL Public Domain Picks reads a short hand-picked list through NDL's [IIIF API](https://dl.ndl.go.jp/ja/help_iiif), which only serves items NDL marks as out of copyright. Those images may be reused without asking; NDL asks to be credited, and is on every chapter. Little Nemo reads its files through the [Commons API](https://www.mediawiki.org/wiki/API:Main_page). Both sources ask again when a chapter is opened and show nothing that has lost its public-domain status. `scripts/check-public-domain.py` runs the same checks over every listed work; run it before a release. Neither source is affiliated with NDL or the Wikimedia Foundation.

## Adding a source

Requirements:

- The rights holder states a free licence (or public domain) for the work. Record the URL. For public domain, check it outside the publishing country too: the author's death date and the publication date.
- Use the official feed or API. No login, no scraping around access controls.
- Write down the attribution the licence requires; it's shown in the app.

Steps:

1. Add the source under `sources/<id>/` and bump `info.version` in `res/source.json`.
2. Add or update its record in `curation/entries.json`, including `payloadSha256` (printed by `scripts/check-curation.sh` in CI).
3. Open a PR. Merging to `main` publishes.

## One-tap add links

A directory's site can link `nemu://addSourceList?url=<index URL, percent-encoded>`; nemu shows the address and asks before adding it. nemu also accepts Aidoku's `aidoku://addSourceList?url=…` and `https://aidoku.app/add-source-list/?url=…`. This directory:

```
nemu://addSourceList?url=https%3A%2F%2Fnemu-pm.github.io%2Fsource-registry%2Findex.min.json
```

## Building locally

```sh
cargo install --git https://github.com/Aidoku/aidoku-rs --rev ae6485e3 aidoku-cli
./scripts/build.sh
./scripts/check-curation.sh
python3 -m http.server -d public 8089
```

## Licence

Code: MIT or Apache-2.0. Content belongs to its authors under the licence listed above. Not affiliated with Aidoku.

To have an entry removed, open an issue.
