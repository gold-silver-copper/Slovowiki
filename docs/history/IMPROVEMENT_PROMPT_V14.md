# Task: V14 — crate 0.13.0, lexicon polish, valence agreement, and pinnable data releases

You are working in `/Users/kisaczka/Desktop/code/slovowiki` (crate `interslavic-wiktionary-lab`). Verified state of master (merge `5f55296`, PR #113): pin `interslavic = "=0.12.0"`, **219 tests green**, translation probe at its recorded baseline **147 verified / 44 generated-only / 28 miss**, evaluate exact top-1 **42.02%** (CI floor 39.5%), form index **569,538 records / 557,118 keys / 77,784 lemmas**, checktext fixture 79 tokens with 0 unknown and 4/4 seeded agreement errors, double export byte-identical, aspect manifest frozen at production both 17.88% / either 48.71% / fingerprint 89.07%.

This brief implements the four items from the downstream (mrzavec) review, **as corrected by investigation** — two of the four comments described a stale slovowiki. House rules at the bottom; corrections of record first, so nobody re-chases phantom bugs.

## Corrections of record (do NOT implement these; cite them in the PR)

- **"Pin is 0.9.0, pronouns come from slovowiki's own STEEN-G tables"** — stale. Master pins 0.12.0; personal/reflexive pronouns have come from the crate's 198/198 parity-verified tables in all three series since PR #112. The STEEN-G supplement (`forms::CLOSED_CLASS`) covers exactly seven function words absent from the dictionary export (`v s k o ob toj ta`) and stays.
- **"The index carries `staje` as prez.3jd. of `stajati`"** — false, verified on both the deployed and current exports: key `staje` holds only the noun `staja` (`gen.jd./nom.mn./akuz.mn.`, official-only); `stajati` is a generated lemma with no inflection records *by documented design* (agent-guide safety property). The row the upstream CHANGELOG references does not exist and never did. The correction belongs upstream; note it in the PR so the claim stops propagating.
- **"`coin-check --lexicon-row` only emits noun rows"** — does not reproduce: `coin-check "mråviti" --gloss "to antify" --lexicon-row` emits `mråviti\tverb\t\t\tto antify` on master (and did since V13). Item 2 adds the regression test + a guide example so this dies documented.

## 1. Pin `interslavic = "=0.13.0"` (trivial, zero-drift)

0.13.0 is a one-item additive patch (`quantified_parts()` — count government as data, for phrase-declining consumers). Its paradigm fingerprint is byte-identical upstream and slovowiki consumes nothing new from it. Bump pin + lock, run the full migration battery (below), and note in the commit that `quantified_parts` was assessed and not adopted for the same reason `quantified` was in #113: steen recommends digits and mrzavec renders digits, so spelled-numeral government gates nothing in the target workload.

## 2. Project-lexicon polish: indeclinable nouns (+ the verb-row regression test)

Real gap (found by mrzavec usage): the lexicon TSV cannot declare a noun indeclinable, so `check-text --lexicon` builds a full — wrong — paradigm for `emu`/`zombi`/`avokado`, and a wrongly-inflected `*emua` would pass the gate as `project`.

- **Format**: `indecl` becomes the third accepted value of the existing **animacy column** (`anim|inanim|indecl`) — the 5-column format, the documented chain (`coin-check → append row → check-text --lexicon`), and all existing lexicons stay valid. Gender stays REQUIRED for indeclinable nouns (adjective agreement still needs it: `zeleny emu`); `indecl` on non-nouns is a parse error like the other noun-only columns.
- **Behavior**: emit the lemma record only — no paradigm records. Consequence to assert, not just implement: `emua` stays `unknown` (that is the point). Declared gender still feeds `Index::noun_gender` via the existing absorbing-homograph pattern, so `zelena emu` (gender mismatch) can still flag.
- **coin-check**: `--animacy indecl` accepted symmetrically; the declinability axis prints "indeclinable — no paradigm" instead of cells; `--lexicon-row` passes `indecl` through. `validate_lexicon_row` skips the declinability probe for indeclinables (nothing to decline) but keeps the collision axis and official-pin checks unchanged.
- **Fixtures/tests**: a lexicon row `emu	noun	m	indecl	emu` (the probe's own monster list supplies the motivation); a text where `emu` is `project` and `emua` is `unknown`; parse rejections (`indecl` on a verb; indecl without gender); the coin-check row round-trip. Plus the item-4b **regression test**: `--lexicon-row` on a verb and on an adjective emits the expected rows (pin the `mråviti` case), and a worked verb example lands in the agent guide's coinage section.

## 3. Valence-aware agreement warnings (the corrected trigger)

The blind spot is real (`hybiš netopyŕa` survived two human passes), but the trigger as proposed — "unambiguous accusative-only noun form" — **cannot catch its own motivating example**: `netopyŕa` is akuz/gen-syncretic (animate masculine). Implement the corrected conservative rule:

- **Valence source**: the official CSV's own `pos_raw` (`v.tr.`/`v.intr.`/`v.refl.`), parsed at `build_index` into `Index::verb_valence: HashMap<String, char>` (`t`/`i`/`r`), keyed by folded lemma, with the SAME absorbing-ambiguity discipline as `noun_gender`: a spelling with both transitive and intransitive official senses absorbs to "abstain". No new data files, no curation.
- **Trigger** (extend `agreement_pass`, same conservatism bar as the existing checks): warn when (a) the verb token is verification-grade and POS-unambiguous (`pure_verb`) and *every* official lemma reading of it is intransitive-only, (b) the adjacent next token is a POS-unambiguous noun, (c) **every** case reading of that noun form is in {akuz, gen} — "object-shaped", which is what catches the syncretic animates — and (d) no punctuation break intervenes. Do not fire on reflexive-tagged verbs (`sę` constructions govern their own cases) or when the noun is also the possible subject (any nom reading escapes via (c)).
- **Hazards to encode as gold sentences BEFORE enabling** (the real cost of this item): genitive-governing intransitives, partitive genitives, negation-genitive, quantified phrases (gen.pl after 5+ numerals — do not fire when a numeral immediately precedes the noun), and postverbal time/measure accusatives (`spal cělų noć` — accusative of duration after an intransitive is GRAMMATICAL; this is the class most likely to burn you. If it cannot be excluded structurally, restrict the check to ANIMATE object-shaped nouns, where duration readings do not occur — state the restriction in the guide).
- **Acceptance**: `hybiš netopyŕa` flags; the checktext fixture, all AGREEMENT_GOLD sentences, and a new valence-gold set (≥8 sentences covering every hazard above) stay at 0 false flags; seeded valence errors (≥3) all flag; `checktext-eval` gains the valence rows in its report. New `TokenReport` field or reuse `agreement` — reuse `agreement` (it is the same class of warning; message names the valence: "glagol 'hybiš' je neprěhodny…"). `--summary` counts stay in `agreement_errors`; no new gate flag.

## 4. Dictionary refresh + pinnable data releases (the deferred #74 — flagship)

V10 already measured live-sheet drift (8 → 17 noun mismatches upstream); mrzavec pins slovowiki by raw commit hash. Make drift a visible, versioned event:

- **`refresh-official` subcommand**: reads a freshly downloaded interslavic-dictionary.com export (path argument — the DOWNLOAD stays a manual maintainer action; no build or benchmark path ever touches the network, house rule 1), normalizes it exactly as `official::load` expects, and writes (a) the new `data/official-isv.csv` and (b) a committed `data/refresh-changelog.md` entry: rows added/removed/changed (id-keyed diff), plus before/after for every headline benchmark (evaluate exact/normalized, corpus-eval, probe verified/generated/miss, aspect both/either/fingerprint, form-index counts). Refuse to write if the diff is empty.
- **Refresh procedure doc** (`docs/DATA-REFRESH.md`): the ordered ceremony — refresh-official → `corpus-eval --fit` (recalibrate) → full `export` (regenerates novel-words) → `aspect-eval` + bless the frozen manifest → re-run the probe and UPDATE `PROBE_BASELINE` with the movement explained in the changelog entry (the "reported metric" design anticipated exactly this) → full validation battery. Every step's output lands in the changelog entry; a refresh PR without the changelog fails CI (add the check).
- **Tagged data releases**: after a refresh (or any consumer-visible artifact change), tag `data-vN` and commit `data/MANIFEST.json`: sha256 + row/record counts for every committed cache and `data/official-isv.csv`, the crate pin, the form-index schema version, and the probe baseline. A selftest asserts the manifest matches the working tree (CI-enforced, so the manifest cannot rot). Downstream pins `data-vN` instead of `bf041ca`. Do NOT auto-tag from CI; tagging is a release decision — document the two-command release ritual in DATA-REFRESH.md.
- **Scope fence**: this item builds the *machinery* and ships `data-v1` for the CURRENT data (manifest + tag + baseline changelog entry, no actual re-pull) — so the tooling is proven without conflating it with a data change. The first real refresh is its own future PR using the documented ceremony.

## Validation (do all, report numbers)

1. `cargo test` green including all new fixtures; clippy `-D warnings`; fmt; all three site validators.
2. Full export; every selftest passes; byte-identical double export; `data/MANIFEST.json` selftest passes.
3. Probe via the tracked runner — items 1–3 touch no data, so it must report **147/44/28** exactly; item 4 ships machinery only, so the baseline is unchanged too.
4. `checktext-eval` reports the valence rows: 0 false flags on fixture + both gold sets, all seeded errors (agreement AND valence) flagged.
5. Demos in the PR: the `emu`/`emua` before/after; `hybiš netopyŕa` flagging with the valence message; `git show data-v1:data/MANIFEST.json` resolving.

## House rules (unchanged)

1. **Deterministic and offline** — `export`/`check-text`/`en`/benchmarks never touch the network; committed caches only; the refresh tool reads a locally supplied file and is never invoked by any build path.
2. **Benchmark honesty** — accuracy paths untouched; a moved probe baseline is legal ONLY in a refresh PR with the movement explained in the committed changelog.
3. **Static-only** — plain files, self-tests, no server.
4. **Contract discipline** — the lexicon TSV stays 5 columns (`indecl` is a value, not a column); consumer-visible shapes kept or schemas bumped; agent guide and README updated in the same commit as each feature.
5. **Reviewable commits** — one per numbered item; honest regressions stated plainly.
