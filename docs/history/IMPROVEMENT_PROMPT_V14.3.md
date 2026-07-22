# Task: V14.3 — fix the ten round-three findings (identity, visibility, honesty)

You are working in `/Users/kisaczka/Desktop/code/slovowiki` on branch `agent/v14-releases-valence` (PR #114, CI green). Round three of the adversarial review surfaced ten findings; the headline is LIVE-CONFIRMED: pinning official `děti` as the noun ("children") hard-fails claiming "the official word is 'verb'". The ten findings collapse into the six work items below. One commit per item.

Baseline to preserve: 228 tests, probe 147/44/28, evaluate 42.02%, `data-manifest` OK (schema 2, release 1), export deterministic. Item 2 makes a DECLARED breaking change to check-text's `--json` shape (the one-time versioned envelope); everything else leaves the export byte-identical — prove it.

## 1. Real entry identity for official records (findings 1 + 4, plus two comment corrections)

`build_index` passes `entry_id 0` for every official record, and `RecordSink` dedups on `(key, lemma_key, entry_id)` — so the official CSV's 41 cross-POS and ~913 same-POS same-surface homographs collapse into one record carrying only the FIRST row's POS/gloss. Item 2 of V14.2 fixed exactly this for proposals and its comment even claims the 1M offset "keeps these ids clear of real official entry ids sharing an index" — no real id ever entered this index. Consequences today: the confirmed `děti` pin failure, `pure_*` purity computed against a masked POS (agreement checks mis-enabled on ambiguous official surfaces), and wrong gloss/POS on ~950 surfaces the site export gets right.

- **Give official records their entry identity in `build_index`**: `entries.iter().enumerate()` → `entry_id = idx + 1` threaded through every `sink.add`/`paradigm_records`/`pronoun_numeral_records` call in the loop (0 stays the closed-class supplement's sentinel; novel rows keep `1_000_000 + i` with a new `debug_assert!(entries.len() < 1_000_000)` making the headroom explicit instead of folklore).
- Fix the two false comments while there: the novel-id comment's "clear of real official ids" becomes true; the V14.2 "this index is CLI-side only, nothing exports it" claim is WRONG twice over — the site export builds this index for the suggest shards (`lemma_keys` are exported; ids are not). Say what is actually true.
- **Consequences to verify, not assume**: `děti` now has verb AND noun lemma records → the pin validates as OfficialPin (THE regression test: `děti	noun	f	anim	children` → OfficialPin); token `děti` carries both readings (assert analyses from both paradigms under the folded key); `pure_*` goes false on such surfaces → agreement ABSTAINS (conservative direction — but rerun `checktext-eval`: the fixture and gold sets must stay 0 false flags AND the seeded sets 4/4 + 3/3. If a seeded sentence stops flagging because its word turns out to be a POS-homograph, replace the word with an unambiguous equivalent and state the swap plainly in the commit).
- **Finding 4's canary**: the V14.2 id change also un-merged a hazard class — a FUTURE novel proposal folding equal to an official lemma of a DIFFERENT POS would now stand beside it and poison purity, silently disabling agreement on the official word. Add the canary test (mirroring the absorbed-gender one): no novel-words row's folded key collides with an official lemma key of a different POS — currently true; a refresh that breaks it must fail a test and force a decision, not silently weaken the checker.
- Export byte-identity: suggest shards carry `[key, lemma]` only — assert the export is unchanged by this item (pinned-env tree hash).

## 2. One versioned JSON envelope — dispositions visible in every JSON mode (findings 2 + 10 + 3)

Three findings, one root: check-text's CLI JSON has no schema version, so V14.2 was forced into "bare `--json` stays a bare array" — which left adoptions with ZERO visibility on the most agent-typical invocation (`--json` without `--summary`), contradicting the guide's fresh claim that "the dispositions are always visible"; and the mandated regression test never shipped (the test rebuilds the envelope shape by hand and cannot catch drift in `run()`'s spliced `format!`).

- **Version it once, now, while the consumer population is one team**: `check-text --json` always emits ONE object `{"schema_version": 1, "tokens": [...], "summary": {...}?, "lexicon": {...}?}` — `summary` present iff `--summary`, `lexicon` present iff `--lexicon`, in BOTH modes. This is a DECLARED breaking change for bare-array consumers: guide and README document the migration in the same commit ("check-text JSON schema 1; the pre-V14.3 bare array is retired"), and the module doc's "--json for agents" sentence points at the version field. `coin-check --json` gains `"schema_version": 1` (additive there — it was already an object).
- **Build it with serde, not splices**: a `#[derive(Serialize)]` envelope struct (named fields; `#[serde(skip_serializing_if = "Option::is_none")]`), rendered by an extracted `fn render_json(...) -> String` that `run()` prints. The `AppliedLexicon`→JSON conversion becomes a `Serialize` shape shared by code and tests — field names exist once.
- **Ship the test that was mandated**: parse `render_json`'s REAL output in both modes — with `--summary`+lexicon: object contains `lexicon.adoptions[0].adopted_gloss`; without `--summary`: object has `tokens` and NO `summary` key; first byte is `{` (no leading prose possible by construction). Delete the shape-copy test.

## 3. Homograph honesty in TokenReport (finding 5)

Same-surface proposals are distinct records now, but `TokenReport` merges them invisibly: `lemmas` dedups by spelling (so `ambiguous` stays false) and `probability` takes the first record's value — presenting aurochs's calibrated probability as "the" probability of a token that may mean prison.

- `ambiguous` becomes true when the surface has MULTIPLE lemma-source records, spelling-equal or not (update its doc from "distinct lemmas" to "distinct lexical readings"); for generated records with differing probabilities, report the MINIMUM (the conservative floor — probability is already documented as never-verification, so under-claiming is the honest direction) — and only when all generated records agree does a single value pass through unchanged.
- Guide documents both rules in the trust section. Test: build a synthetic proposals file with two same-surface probabilities (0.62/0.31) → token reports `ambiguous: true`, `probability: 0.31`. With item 1 landed, also assert official `děti` reports `ambiguous: true` now.

## 4. coin-check shows WHICH concept (finding 6)

The collision axis dedups on `(lemma, pos, status)` with no gloss, collapsing the two `tur` concepts into one gloss-less row; and `--lexicon-row` discards the `adopted_gloss` the disposition carries — the pre-commit tool cannot say which concept a row would adopt, though check-text's envelope can.

- Collision records gain a `gloss` field (dedup key extends to include it; additive JSON; human rows print a truncated gloss like the false-friend lines do).
- `--lexicon-row` output: human line appends `(adopts proposal glossed '…')` for adoptions; `--json` gains `adopted_gloss` beside `lexicon_row_disposition`. Test: `coin-check tur` lists BOTH concepts; the row hand-off for the prison gloss reports the prison proposal.

## 5. Manifest flag discipline and a self-check that isn't circular (findings 7 + 8, plus the schema-1 migration wart)

- **`--release` without `--write` is an error** (`ensure!` up front): today verify mode renders with the passed N, fails with the canned data-drift message, and the message's prescribed recovery silently loses the intended bump. Fix the two error texts that say "regenerate with `--release N`" to say `--write --release N`.
- **`data_release` must not be self-attested**: verify mode currently reads N from the manifest under verification — a hand-edit round-trips green everywhere until a tag push. Add an offline cross-check inside `data-manifest`: the newest `### data-vN` heading in `data/refresh-changelog.md` must equal the manifest's `data_release` (two files must now be edited consistently to lie; both are committed and reviewed). Keep the tag-name check in `data-release.yml` as the final authority. Tests: mismatched heading fails verify with a message naming both numbers; the wart from round three's audit — `--write` on a schema-1 tree demands `--release` with an error text that never mentions `--write` — gets the corrected text plus one sentence in DATA-REFRESH.md's ritual ("first run on a pre-schema-2 tree: `--write --release N`").

## 6. One eligibility predicate (finding 9)

`build_index` re-implements the masc-animate filter inline while the site path calls `masc_animate_lemma_keys` — only the maps are shared, not the predicate, and the helper's "can never disagree" doc overstates. Refactor: `masc_animate_from_maps(&gender, &animate) -> HashSet<String>` holds the predicate ONCE; `masc_animate_lemma_keys(entries)` wraps it; `build_index` calls the maps variant on its own maps. (This also removes the site path's duplicate `noun_trait_maps` recomputation flagged by the efficiency pass — accept the free win, don't chase the rest of that family here.)

## Validation (do all, report numbers)

1. `cargo test` green (new: děti pin regression, both-readings assertion, novel-vs-official POS canary, real-output envelope tests both modes, homograph probability/ambiguous, coin-check tur listing, manifest cross-check + flag rejection); clippy `-D warnings`; fmt; three site validators.
2. **Export byte-identical under pinned env vs the pre-V14.3 head** (items 1, 3–6 are CLI-side; item 2 changes only CLI stdout) — tree-hash proof.
3. Probe **147/44/28**; evaluate **42.02%**; `checktext-eval` fixture + all gold sets 0 false flags, seeded 4/4 + 3/3 (with any homograph-forced sentence swap stated plainly); `data-manifest` OK including the new changelog cross-check.
4. Live re-runs: `děti	noun` pin validates; `check-text --json --lexicon` (no `--summary`) parses AND contains `lexicon`; `coin-check tur` shows both glosses.
5. PR comment maps findings→commits; the round-three findings list gets outcomes (fixed / conceded-with-reason).

## House rules (unchanged)

Deterministic/offline; benchmark honesty (no accuracy path touched); static-only; contract discipline — item 2's envelope is the PR's one declared breaking change, versioned and documented where consumers look, in the same commit; reviewable commits, one per item; honest regressions stated plainly.
