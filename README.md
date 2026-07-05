# Interslavic Wiktionary Lab

An **evidence-based Interslavic (Medžuslovjansky) candidate-generation engine** with a
reproducible accuracy benchmark against the official Interslavic dictionary, plus a
local Wiktionary-style website that shows, for every meaning, the generated candidate,
its rule trace, the Slavic evidence by branch, a calibrated confidence, and whether it
matches the official dictionary.

No SQLite / database. No hotlinked Wikimedia CSS/JS. Everything is native Rust with an
in-memory index.

## Core principle

> No algorithmic change is kept unless it improves **measured accuracy** on the
> reproducible benchmark against official Interslavic data.

Every rule is gated behind a flag and measured in isolation on an ablation ladder.
Rules that regress accuracy are reverted and documented (see the *rejected experiments*
in the report).

## Results (production config vs. original prototype)

Benchmark: reconstruct the official Interslavic lemma from the modern Slavic cognates in
the official dictionary, **without ever showing the generator the answer**
(16,300 single-word entries).

| Metric | Baseline (prototype) | Production | Δ |
|---|---:|---:|---:|
| exact top-1 | 27.38% | **32.42%** | +5.04 pp |
| normalized top-1 | 34.96% | **40.43%** | +5.47 pp |
| normalized top-3 | 42.89% | **49.2%** | +6.3 pp |
| mean normalized edit distance | 0.253 | **0.238** | −0.015 |

**Confidence calibration** (high-confidence candidates match far more often — as intended):

| confidence | n | normalized match |
|---|---:|---:|
| high | 4,601 | 67% |
| medium | 9,410 | 35% |
| low | 2,289 | 10% |

Full metrics, POS-specific accuracy, branch-coverage analysis, regression/improvement
lists and the remaining-error breakdown are regenerated into `target/eval/` (a committed
snapshot is under version control).

## What was kept (each improved measured accuracy)

1. **Branch-balanced consensus** — vote on a consonant-skeleton alignment key counting
   Slavic *branches*, not languages, so Russian/Polish can't dominate.
2. **Six-subgroup vote** (§4.1 of the rule spec) — one vote each for RU / UK+BE / PL /
   CZ+SK / SL+HR+SR+BS / BG+MK, with population as a tie-break only.
3. **POS lemma endings** (§3) — noun nom.sg, adjective `-y`/`-i`, verb infinitive `-ti`.
4. **Internationalism table** (§5.2) — `-ism→-izm`, `-tion→-cija`, `-ic/-ical→-ičny`,
   `-al→-alny`, `-ive→-ivny`, verbs→`-ovati`, plus `au→av`, `eu→ev`.
5. **Prefix normalization** — `roz-/ras-/raz-/ros- → råz-`, `pred- → prěd-`.
6. **De-pleophony** (liquid metathesis) and **nasal recovery** (`ę/ų` from Polish).
7. **g-preserving representative** — Interslavic keeps *g, so g-languages outrank the
   Czech/Slovak *g→h forms when picking the surface.

## What was rejected (regressed the benchmark)

Recovering flavored letters (`ć/đ`, jat `ě`, `*y`) from *modern reflexes* is too noisy —
each experiment regressed accuracy. The rule spec's own prescription (§4.4) is to derive
those from a **Proto-Slavic reconstruction** once consensus has chosen the root; that is
the top item in "next recommended rules".

## Architecture

```
src/
  model.rs         Candidate / RuleStep / Evidence / Confidence / MatchStatus / Pos
  lang.rs          Slavic language + branch + script metadata
  normalize.rs     per-language script → common phonemic Latin (keeps ě/ę/ǫ/č/ć/đ)
  orthography.rs   flavored↔standard folding, ASCII skeleton, edit distance
  official.rs      official dictionary loader (quote-aware CSV / TSV)
  consensus.rs     branch-balanced modern-Slavic consensus engine (gated rules)
  morph.rs         POS lemma endings + internationalism ending table
  proto.rs         Proto-Slavic → Interslavic ordered rule engine (+ tests)
  overrides.rs     manual curation (TOML), excluded from pure-algorithm accuracy
  generator.rs     orchestrator: consensus + proto + overrides + match status
  eval.rs          reproducible benchmark, ablation ladder, report writers
  site.rs          build + serve the local Wiktionary-style website
data/
  official-isv.csv   the full official dictionary (evidence + gold, self-contained)
  overrides.toml     manual curation file
  RULE_SPEC.md       authoritative Proto-Slavic → Interslavic rule specification
```

## Commands

```bash
# Reproducible benchmark against the official dictionary (fast, no dump needed):
cargo run --release -- evaluate --official data/official-isv.csv --out target/eval

# The acceptance-criteria invocation also works (the metadata TSV lacks
# translations, so it transparently falls back to the bundled full export):
cargo run --release -- evaluate \
  --dump /Users/kisaczka/Desktop/code/english/raw-wiktextract-data.jsonl \
  --official /Users/kisaczka/Desktop/code/interslavic-rs/crates/interslavic/data/dictionary_metadata.tsv

# Build the website dataset and serve it:
cargo run --release -- build --dump /Users/kisaczka/Desktop/code/english/raw-wiktextract-data.jsonl
cargo run --release -- serve            # http://127.0.0.1:8765

# Explain one word/gloss (manual spot-check with full rule trace):
cargo run -- explain duša
cargo run -- explain "computer"
```

## Website

Each entry page shows:

- the **top candidate** headword with calibrated **reliability**;
- **alternative** candidates with scores and branch coverage;
- the **rule trace** (each transformation, before→after, with a doc citation);
- the **evidence by Slavic branch** (East / West / South), linking back to Wiktionary;
- the **official-dictionary match status**: *officially attested* / *differs from
  official* (both shown) / *no official entry*;
- full **inflection tables** generated by the local `interslavic` crate.

## Benchmark artifacts

```
target/eval/candidate-generation-summary.json   per-rung metrics (machine-readable)
target/eval/candidate-generation-report.md      full human-readable report
target/eval/regressions.csv                      matched before, not after
target/eval/improvements.csv                     newly matched
target/eval/errors-sample.csv                    nearest remaining misses
```

## Provenance / license note

Slavic evidence and the official lemmas come from the Interslavic dictionary
(interslavic-dictionary.com) and English Wiktionary/Wiktextract. Generated data keeps
source URLs; a public deployment needs a proper attribution/license page.
