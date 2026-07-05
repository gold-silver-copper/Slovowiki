# Interslavic Wiktionary Lab

Rust prototype pipeline and local in-memory website for generating an Interslavic-first Wiktionary from English Wiktionary / Wiktextract data.

This version:

- streams the raw English Wiktextract JSONL dump;
- extracts **all configured Slavic languages** plus Proto-Slavic, Proto-Balto-Slavic, and Proto-Indo-European lexemes;
- generates Interslavic candidate entries from **all Proto-Slavic pages** by default;
- flattens cited Slavic descendant trees into source tables;
- writes one JSON data artifact;
- launches a local server that loads the full artifact into native Rust structs and indexes in memory;
- renders entries with local Wiktionary/MediaWiki-inspired HTML and CSS classes;
- does **not** hotlink Wikimedia CSS/JS;
- does **not** use SQLite or any database server.

## Quick start

```bash
cd /Users/kisaczka/Desktop/code/interslavic-wiktionary-lab
cargo run -- build --max-proto 200 --max-lexemes 10000
cargo run -- serve
```

Open:

```text
http://127.0.0.1:8765
```

A small sample artifact has already been built at `data/wiktionary-lab.json` so `cargo run -- serve` works immediately.

## Full build

By default, `build` scans the whole dump and has no limits:

```bash
cargo run --release -- build --dump /Users/kisaczka/Desktop/code/english/raw-wiktextract-data.jsonl
```

Equivalent Make target:

```bash
make full
```

This can take a while because the raw dump is large. For development, use caps:

```bash
cargo run -- build --max-proto 500 --max-lexemes 50000
```

## Commands

```bash
cargo run -- build [--dump PATH] [--output data/wiktionary-lab.json]
cargo run -- serve [--data data/wiktionary-lab.json] [--port 8765]
```

Or:

```bash
make build   # quick capped development build
make full    # all Slavic/proto lexemes and all Proto-Slavic generated entries
make serve   # local server
```

## Wiktionary-style pages

Entry pages are rendered with MediaWiki-like classes such as `mw-parser-output`, `mw-headline`, `toc`, and `wikitable`, styled by local CSS in `static/wiktionary.css`. The site does not depend on Wikimedia-hosted CSS or JavaScript.

Each generated entry page includes:

- Interslavic section heading;
- generated-entry confidence warning;
- etymology section;
- part-of-speech section;
- cognate/descendant `wikitable`;
- reference list linking back to English Wiktionary.

## Current candidate algorithm

The generated candidate form is intentionally marked as a first-pass candidate, not final linguistic truth. It currently applies a visible Proto-Slavic-to-Interslavic approximation:

- strip reconstruction marker `*`;
- remove stress/accent notation;
- approximate final yers by dropping them;
- approximate internal yers as `e`/`o`;
- normalize a few orthographic conventions.

The main value of this first version is the citation graph: every generated entry includes Wiktionary links and descendant evidence.

## Next steps

- Replace the placeholder Proto-Slavic → Interslavic candidate function with a real rule engine.
- Add official Interslavic dictionary entries as seeds/overrides.
- Add manual review/curation files.
- Add static-site export.
- Add a license/attribution page before public deployment.

## License note

Wiktionary-derived content has license obligations. This prototype keeps source URLs and dump provenance in generated data, but a public deployment needs a proper attribution and license page.
