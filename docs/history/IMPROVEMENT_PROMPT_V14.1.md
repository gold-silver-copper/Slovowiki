# Task: V14.1 — fix the ten PR #114 review findings, at the right altitude

You are working in `/Users/kisaczka/Desktop/code/slovowiki` on branch `agent/v14-releases-valence` (PR #114, currently **red** — finding 1 below is why). The ten findings come from an 8-angle adversarial review; six are CONFIRMED (three by live reproduction, one by the failing CI run itself). Fix them **on this branch** as review-hardening commits — grouped as the seven work items below, because three findings share one root cause and two pairs share one proper fix. Do not open a new PR.

Baseline to preserve: 223 tests, probe 147/44/28, evaluate 42.02%, byte-identical double export, aspect manifest frozen. The item-2 fix changes check-text analyses (deliberately, see below) but must NOT change the exported site — assert that explicitly.

## Item 1 — CI: make the changelog guard shallow-clone-safe (finding 1, CONFIRMED by the red run)

`git diff origin/master...HEAD` (three-dot) needs a merge base; the PR checkout is a depth-1 merge commit with grafted parents, so the diff exits 128, `2>/dev/null` hides the fatal, `!` misreads 128 as "CSV changed", and the step fails EVERY pull request with a misleading message (run 29941979803).

**Proper fix — two-dot tree comparison, not a deeper fetch**: `git diff origin/master HEAD` compares trees directly, needs no merge base, and on a PR merge commit vs the fetched master tip yields exactly the PR's changes. Keep the `git fetch origin master --depth=1`; drop BOTH `2>/dev/null` suppressions (a git failure must fail the step loudly as itself, never masquerade as a policy violation); keep the `rev-parse` guard for master pushes where the diff is empty anyway. Do NOT use `fetch-depth: 0` — the repo's data history makes full clones expensive for nothing. Acceptance: CI green on this PR; a scratch branch touching `data/official-isv.csv` without the changelog still fails the step (verify locally by replicating the step's shell against a `--depth=1` clone of your own repo — the reviewer reproduced the bug that way, reproduce the fix the same way).

## Item 2 — animacy at the record layer (findings 2 + 4, both CONFIRMED live; one root cause)

Two symptoms, one hole: official animate nouns decline inanimate (deliberate — cell SURFACES stay as-is), so their gen-shaped accusatives carry no `akuz` analysis. The valence check papered over it with the `animate_gen_sg` special case (which finding 2 shows also mis-fires: `akuz_gen_both` lacks the number guard its own comment claims, flagging grammatical `Pribyvaje žabervokov`); the sibling preposition-government check consumes the same broken analyses and false-flags `Gledaš črěz netopyŕa`.

**Proper fix — enrich the ANALYSES in `build_index`, not the consumers, and not the paradigm cells**:

- After records are indexed, one enrichment pass: for every noun record whose lemma is dictionary-animate (`noun_animate == 'a'`) AND masculine (`noun_gender == 'm'`; feminine a-stems have a distinct accusative, neuter animates decline nom-like — neither is enriched), append the parallel accusative analysis to genitive-bearing analyses: `gen.jd.` gains `akuz.jd.`, `gen.mn.` gains `akuz.mn.`. Surface strings, keys, and the exported site are untouched — this is a check-text/coincheck-index statement about READINGS, which is exactly what the animate accusative-genitive syncretism is. Document the rule where `noun_animate` is declared and in the agent guide (the CSV tag is trusted for *readings and warnings*, never to reshape *cell surfaces* — resolve the stated-rationale tension in one sentence there).
- Then DELETE the `animate_gen_sg` branch entirely: enriched official forms (`netopyŕa` → `{gen.jd, akuz.jd}`) now satisfy `akuz_gen_both` like project nouns do — one rule, no special case.
- Add the missing number guard to the now-single branch: fire only when every reading is SINGULAR (`f.number == 'j'`) — the comment "plural genitives are excluded: partitives live there" becomes true.
- Fixtures: gold gains `Gledaš črěz netopyŕa.`, `Na vojakov!` (or another akuz-governing prep + animate plural), and `Pribyvaje žabervokov.` (checked WITH the committed lexicon — extend the eval/test plumbing to run the valence sets against a lexicon-loaded index so project plurals are covered); seeded errors stay 3/3, `checktext-eval` fixture stays 0 false flags.
- Acceptance: both reproduced sentences verify clean; `Hybiš netopyŕa.` still flags; **the export is byte-identical to the pre-fix branch** (the enrichment lives in `build_index`, which the site path does not use — prove it with the pinned-env double-export hash).

## Item 3 — valence absorb gate + shared animacy vocabulary (findings 5 + 10)

- Finding 5 (CONFIRMED mismatch): `build_index` gates `absorb_insert` on `v != ' '`, so aux/untagged senses never poison a tagged one — `iměti` (v.tr. + v.aux. senses) records `'t'` while both doc comments promise abstention. **Drop the gate** — `absorb_insert`'s `and_modify` already poisons correctly when fed `' '` (trace it, then pin it: a unit test asserting `iměti`'s valence key is `' '`, and one asserting a pure `v.intr.` lemma still reads `'i'`). Currently harmless (only `'i'` fires; zero intr+untagged homographs in today's data — state that in the commit), but `refresh-official` makes new data routine and this is the class of latent bug the absorb discipline exists to prevent.
- Finding 10 (CONFIRMED, already diverged once inside PR #114): the `""|anim|inanim|indecl → (Option<bool>, bool)` mapping lives as twin matches in `parse_lexicon` and `Overrides::parse`. Extract one `pub fn parse_animacy(raw: &str) -> Result<(Option<bool>, bool)>` in check.rs (error text lists the legal values once); both callers add their own context ("lexicon line {n}" / "--animacy"). The coin-check→check-text row contract is now un-drift-able by construction.

## Item 4 — refresh parser on the real CSV reader (finding 3, CONFIRMED semantics)

`raw_rows` re-implements CSV row splitting with three latent corruptions: continuations glue to `values_mut().last()` (max STRING key, not the previous row — ids are unsorted and `"9998" > "24021"`), comma-less continuation lines vanish (a real refresh can be refused as "no-op"), and a digit-comma continuation line clobbers an unrelated row. `official::read_csv_records` is `pub` and RFC-4180-correct, and `run_refresh` already trusts it via `official::load`.

**Proper fix**: delete the heuristic; build both id→row maps from `read_csv_records(&text)` — id = record cell 0, value = the remaining cells re-joined with a non-CSV separator (`\u{1f}`) so comparison is content-exact and quoting-insensitive. Unit test with a synthetic CSV exercising all three old failure modes: a quoted multiline cell, a comma-less continuation line, and a continuation line starting `1985,` — assert the diff names exactly the truly-changed id. Duplicate ids in one file (last-wins vs error): make it an error — a refresh input with duplicate ids is corrupt.

## Item 5 — manifest hardening (findings 6 + 7)

- Finding 6: render `MANIFEST.json` via `serde_json` (already a dependency in ten modules) with `to_string_pretty` — escaping and well-formedness for free; verification stays byte-exact against the canonical serde rendering. Read the pin from **Cargo.lock** (`[[package]] name = "interslavic"` → its `version`), which is the resolved truth and format-stable, instead of trim-matching a Cargo.toml line that legal formatting (`{ version = "…" }`) breaks. Record it as `"=X.Y.Z"` to preserve the field's meaning.
- Finding 7: derive the covered set from `git ls-files -z -- data` (tracked files are the authority; delete `MANIFEST_EXCLUDE` and its already-wrong .gitignore mirror). Run git via `std::process::Command`; a missing git or non-repo dir is a hard, clearly-worded error — this tool is maintainer/CI-facing. Restructure so the hashing/rendering core takes an explicit file list, keeping the tamper/roundtrip unit tests git-free; the `committed_manifest_matches_tree` test keeps using the git-derived list (tests run at the crate root). Acceptance: a stray `data/scratch.csv` no longer affects `data-manifest` in either direction; regenerate and commit the manifest (its bytes change with the serde rendering — that diff is this item's visible event; MANIFEST_SCHEMA stays 1, the shape is unchanged, only formatting).

## Item 6 — make adoption explicit and semantically guarded (finding 8)

Adoption currently happens silently (no output anywhere; coin-check discards the disposition) and matches on folded key + POS only, so a data refresh emitting an unrelated same-surface proposal silently flips a Coinage row into adopting the wrong concept — where pre-PR behavior was a hard error forcing review.

**Proper fix, three parts**:
- **Surface equality**: adoption requires `r.lemma == row.lemma` exactly, not folded-equal — a spelling mismatch is a rejection naming both spellings ("adopts proposal spelled 'X'; match it or coin another surface"). This also moots the stale-display-lemma nuance in `lemma_keys`.
- **Semantic guard**: require non-empty gloss-token overlap (`english_gloss_tokens`, the machinery the consistency check already uses) between the row's gloss and the adopted proposal's gloss; reject otherwise, quoting the proposal's gloss — the concept-flip scenario becomes a loud error instead of a silent adoption.
- **Signal**: `apply_lexicon` returns (or exposes) per-row dispositions; `check-text --lexicon` prints a one-line load summary ("lexicon: 5 rows — 3 coinages, 1 official pin, 1 adoption (emu ← 'emu bird')"); coin-check's `--lexicon-row` prints the disposition in the human report and adds a `lexicon_row_disposition` field to `--json`. Update the adoption test (synthetic proposals file) for all three parts and the agent guide's adoption paragraph.

## Item 7 — README valence documentation (finding 9)

Add the valence check to README's warning-taxonomy paragraph (lines ~571) with one sentence matching the agent guide's: intransitive-only per the dictionary's own `v.intr.` tag, object-shaped singular animate noun, `ne` abstains. House rule 4 wanted this in the item-3 commit; say so plainly in this commit's message instead of pretending otherwise.

## Validation (do all, report numbers)

1. `cargo test` green (new: iměti-abstain, animacy round-trip via shared parser, multiline-CSV refresh diff, adoption surface/gloss/signal, enrichment analyses); clippy `-D warnings`; fmt; three site validators.
2. **Export byte-identical to the current branch head** under the pinned env (item 2's enrichment must not leak into the site); byte-identical double export; `data-manifest` OK after the item-5 regeneration.
3. Probe 147/44/28; evaluate 42.02%; aspect manifest and calibrations zero-drift; `checktext-eval` reports agreement 4/4, valence 3/3, and ALL gold sets (including the three new sentences) at 0 false flags.
4. CI green on PR #114 — the reproduced failures (`Gledaš črěz netopyŕa.`, `Pribyvaje žabervokov.` with lexicon, the guard step) each shown fixed in the PR discussion, and the ReportFindings outcomes updated (`fixed` per finding; finding 9's same-commit breach recorded as fixed-late).

## House rules (unchanged)

Deterministic/offline; benchmark honesty (no accuracy path touched — the enrichment is a check-text reading, not a generation change); static-only; contract discipline (guide + README in the same commit as each behavior change — including the enrichment rule and the adoption guards); reviewable commits, one per item above; honest regressions stated plainly.
