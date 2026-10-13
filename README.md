# nemu source registry

The recommended source directory built into [nemu](https://nemu.pm).

```
https://nemu-pm.github.io/source-registry/index.min.json
```

Only works whose rights holder publishes them under a free licence are listed, plus sources that only reach a server you run. Details for each entry are in [`curation/entries.json`](curation/entries.json).

## Sources

| Source | Languages | Licence | Author |
| --- | --- | --- | --- |
| Pepper&Carrot (`multi.peppercarrot`) | en, ja, zh, fr, de, es, it, pt, ru, ko | [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) | [David Revoy](https://www.peppercarrot.com) |
| LANraragi (`multi.lanraragi`) | multi | — (your own server) | [LANraragi](https://github.com/Difegue/LANraragi) / [Aidoku Community](https://github.com/Aidoku-Community/sources) |

LANraragi has no content of its own; it connects to a server you run.

Pepper&Carrot reads the site's official [API](https://www.peppercarrot.com/ld/documentation/130_API.html). Translators are credited on each chapter. The author's [2026-10-08 note](https://www.davidrevoy.com/article1178/license-update-ai-derivation-prohibited-on-all-my-art-lore-stories-and-comics) prohibits AI-generated derivatives; nemu doesn't make any. We'll remove the entry if he asks.

## Adding a source

Requirements:

- The rights holder states a free licence (or public domain) for the work. Record the URL.
- Use the official feed or API. No login, no scraping around access controls.
- Write down the attribution the licence requires; it's shown in the app.

Steps:

1. Add the source under `sources/<id>/` and bump `info.version` in `res/source.json`.
2. Add or update its record in `curation/entries.json`, including `payloadSha256` (printed by `scripts/check-curation.sh` in CI).
3. Open a PR. Merging to `main` publishes.

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
