# Task: V12 — Slavic-evidence prefer, calibration depth, and coinage tooling

You are working in `/Users/kisaczka/Desktop/code/slovowiki` (crate `interslavic-wiktionary-lab`). V11 (archived at `docs/history/IMPROVEMENT_PROMPT_V11.md`, commits `7ffaf9a..e30fc4d`) delivered the precision pass: false-friend FP classes eliminated (3,947→3,302 notes with severity), fail-closed POS-gated `prefer`, ranking tie-breakers + the sense note, `-ovati` verb completions (post-review gated 127→36), both calibrators fitted, sharded notes, and `en --batch`. A post-merge review verified all of it (219-word game probe stable at 147 verified / 44 generated / 28 miss; `teleportovati` resolves; the `staff` sense note fires; all V11 fixtures pass; 204 tests green).

V12 targets the residuals that review found, plus the next capabilities translation work needs. Every item cites observed evidence — turn each observation into a regression fixture. House rules at the bottom are unchanged and binding.

## 1. `prefer` — replace English-token scoring with Slavic-side evidence

The V11 threshold raise helped, but the surviving bad prefers are all the same disease: **English polysemy fully covering a divergent sense**, which no ppm threshold can fix:

- `ravnina` (trap sense: 'plane (surface)') → prefers `aviakarta` — via English "plane" the aircraft.
- `skloniti` (trap: pl 'to prompt/induce') → prefers `opadati` — via English "decline".
- `gojiti` (trap: 'to fatten') → prefers `dobyti`; `staja` (trap: 'stable/barn') → prefers `komnata` — weak single-token grazes that slipped the gate.

Meanwhile the *correct* prefers (`urok`→`lekcija`, `pytati`→`nakrmiti`, `čas`→`časina`, `barviti`→`harakterizovati`) show what the feature should do. Fix it with evidence from the *Slavic* side instead of English tokens: the colliding word's own cross-language links are in the caches (wiktextract `translations`/`synonyms` for the colliding Slavic word; the official dictionary's per-language cognate cells). A candidate prefer lemma should be supported by a **Slavic-language bridge** — the colliding word's translations/synonyms intersect the prefer lemma's own attested forms or cognate cells — with English token coverage demoted to a secondary signal. Where no bridge exists, emit empty (fail-closed stands). Fixtures: the four bad prefers above must go empty or sensible; the four good ones must survive.

## 2. Warning rendering — exact collisions outrank loose

Observed: the `pytati` note's rendered warning quotes only ru питать 'to feed' (a **loose** y→i-fold collision) while ru пытать 'to torture' — the **exact**-fold collision and the classic trap — sits unrendered in the machine-readable `collisions` list. A reader gets the mild warning and misses the severe one.

Fix: when selecting which colliding words a warning sentence quotes (per language, up to `MAX_WARNED_LANGS`), order exact-level collisions before loose-level, then by severity of divergence — and if both levels exist for one language, prefer quoting the exact one. The full `collisions` list stays complete and level-annotated (add a `level: exact|loose` field if absent). Fixture: `pytati`'s rendered warning must mention the пытать 'torture' sense.

## 3. Novel-word proposals — reconcile near-official byforms (with issue #99)

`data/novel-words.tsv` is live again (286 rows, all in the review band). Row 2 is `jabluko` — "novel", 9 languages, 3 branches… but the official dictionary has `jablȯko`. That is not a novel word; it is a **near-miss byform of an official lemma** (the reconstruction disagreeing with the official form by one vowel grade). Shipping it as a proposal misstates what the pipeline found.

Add a reconciliation pass over proposals: a proposal whose gloss matches an official row (gloss-token/POS match, same machinery the calibrator holdout uses) and whose folded form is within small edit distance of that row's lemma (or any of its comma-separated byforms — this is where open issue **#99**, byform consistency, joins in) is reclassified `near-official` with the official lemma cited — a *reconstruction-mismatch diagnostic*, valuable for tuning sound rules, not a proposed word. Report the split (truly-novel vs near-official) in the proposals artifact and README; fixture: `jabluko` classifies as near-official against `jablȯko`. While in #99's territory: make comma-separated official byforms consistent between site pages and API records per that issue's description.

## 4. Corpus calibration — break the flat ceiling honestly

The isotonic decile map is well-calibrated (holdout ECE 0.0142) but **saturates at ≈0.43**: every one of the 286 proposals lands at 0.428, because a single coverage score cannot separate a 9-language/3-branch set (`jabluko`) from a 4-language one — both sit in the top decile. Nothing can ever reach the 0.6 propose threshold, making the proposal pipeline decorative.

Extend the calibrator's feature space the same way the borrowing calibrator already works: bucket by (languages, branches, borrowed, gloss-agreement) — or coverage-decile × langs-band — with per-bucket Wilson-95 lower bounds on the same dev/holdout split, PAVA within bands to keep monotonicity, machine-checked domain as now. Two honest outcomes are acceptable: strong buckets rise above 0.6 and the proposal band becomes real, or the data shows even 9l/3b reconstructions only match official decisions ~half the time and the README documents *that* with numbers. Either way the flat-0.428 column disappears. Keep the benchmark path byte-stable; this also inherits the deferred cleanup — unify the duplicated PAVA/fit helpers between `calibrate.rs` and `eval.rs`'s inline fit (noted in the V11-5 commit).

## 5. Verb completions — close the sibling-lexeme residue

V11's post-review gate (127→36, all sampled clean) left one admitted residue: prefix-sharing sibling lexemes (`transljacija` → glossed 'to translocate' via the transl- family). The current gate checks a 4–6 char skeleton *prefix*; siblings share exactly that prefix. Tighten: require the attesting verb's stem skeleton to match the noun's stem skeleton in full (minus the derivational tail), not merely by prefix — `translj-` ≠ `translok-`. Fixture: `transljacija` either gets a translation-family gloss or drops out; `teleportovati`, `organizovati` intact.

## 6. `coin-check` — validate coined words (the translation-facing feature)

Real translation work (the mrzavec game) has ~8 unavoidable coinages — fantasy names (`jabberwock`, `xeroc`, `aquator`) with no dictionary answer anywhere. Right now a coiner gets no tooling. Add `cargo run --release -- coin-check <word> [--json]`, entirely from existing machinery:

1. **Phonotactics**: legal ISV alphabet + cluster check derivable from the corpus itself (attested bigram/trigram inventory over official lemmas — deterministic, no hand list); flag un-Slavic sequences and illegal letters.
2. **Collision**: fold the word, look it up in the form index (must not collide with an existing lemma/form — or report what it collides with).
3. **False-friend risk**: run the V10/V11 collision machinery for the coined surface across the 10 languages' caches — report any language whose speakers would read the coinage as an existing word, with glosses.
4. **Declinability**: report the gender/animacy/paradigm the `interslavic` crate would guess from the ending, so the coiner knows how it will inflect (and can adjust the ending to get the paradigm they want).

Output: pass/warn per axis, `--json` for agents. This is the last missing piece between slovowiki and a fully-tooled game translation. Selftest with one known-good official lemma (all-pass), one deliberate collision, one illegal cluster.

## 7. (Smaller, do last) `en --batch` polish from first real use

- Multiword queries report per-content-word fallback hits; add the originating word to each hit in `--json` batch mode (parity with single-query mode's `key`) — a lexicon builder mapping "poison dart trap" needs to know which token produced which candidate.
- The batch summary should count sense-note occurrences (`sense_notes: N`) so an agent knows how many results need gloss reading.

## Validation (do all, report numbers)

1. `cargo test` all green including new fixtures (items 1, 2, 3, 5, 6).
2. Full export; all selftests (forms, en, notes, suggest + the new coin-check selftest) pass against the fresh export.
3. Re-run the 219-word game probe with `en --batch` (V11 result: 147 verified / 44 generated-only / 28 miss) — items 1–5 must not regress it.
4. Re-sample 18 notes (seed 42, sorted keys): report prefer quality (V11 result: ~4 bad prefers in sample) and confirm the `pytati` warning quotes the torture sense.
5. Report the novel-words split (novel vs near-official) and the new probability distribution (V11: all 286 at 0.428).

## House rules (unchanged)

1. **Deterministic and offline** — committed caches only; no network at export; no ML models; fixed thresholds; explainable string algorithms.
2. **Benchmark honesty** — pure-algorithm accuracy paths untouched; calibrators leakage-free and holdout-validated; curated knowledge only as test expectations.
3. **Static-only** — plain files, self-tests, no server.
4. **Contract discipline** — keep consumer-visible shapes or bump schemas; update `agent_guide()` and README in the same commit as each feature.
5. **Reviewable commits** — one per numbered item; honest regressions stated plainly.
