# Contributing

This project generates a fully static, Wiktionary-style Interslavic dictionary from evidence caches.

## Local workflow

```bash
cargo test
cargo run --release -- export --out site
cd site && python3 -m http.server 8765
```

Then open <http://localhost:8765>.

## Report or curate an entry

Every generated entry has a **Popraviti / issue** tab that opens a pre-filled GitHub issue.
For local curated notes, create `data/curation-notes.json`:

```json
{
  "voda": "Checked manually: matches official form and broad Slavic evidence.",
  "123": "Entry-id notes are supported, but headword keys are more stable."
}
```

Headword keys are folded to the site's standard spelling. Numeric IDs work too, but can change after regeneration.

## Useful commands

- `make export` — regenerate `site/`
- `make serve` — regenerate and serve locally
- `make test` / `cargo test` — run tests
- `make corpus-eval` — evaluate the corpus-driven site path
- `make eval` — official dictionary benchmark

## Static wiki pages

The exporter also writes generated wiki navigation pages:

- `all-pages.html`
- `categories.html` and `category/*.html`
- `indices.html` and `index/*.html`
- `portals.html` and `portal/*.html`
- `graph.html` / `graph.json`
- `what-links-here/*.html`
- `sitemap.xml`
- `build.json`
