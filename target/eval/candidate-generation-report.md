# Candidate-generation benchmark

Benchmark: reconstruct the official Interslavic lemma from the modern Slavic cognates in the official dictionary, **without showing the generator the answer**. Evaluated on **16300** benchmarkable single-word entries. Every rule is kept only if it improved measured accuracy.

- **Metrics.** *exact*: identical to the official flavored lemma; *normalized*: identical after reducing both to the standard alphabet (§1.3); *skeleton*: identical after an ASCII fold; *top-3/5*: any of the first N candidates matches (normalized); *mean edit*: mean normalized Levenshtein distance to the official lemma.

## Kept rules — cumulative ablation ladder

Each rung adds exactly one rule to the previous, so its accuracy delta is attributable. The last rung is the kept **production** configuration.

| Rung | exact top-1 | norm top-1 | Δ norm | top-3 | mean edit |
|---|---:|---:|---:|---:|---:|
| baseline | 27.52% | 35.23% | +0.00 pp | 43.26% | 0.252 |
| +branch-consensus | 28.06% | 36.28% | +1.04 pp | 44.68% | 0.251 |
| +six-subgroup | 28.30% | 36.55% | +0.27 pp | 44.47% | 0.251 |
| +lemma-endings | 30.10% | 38.76% | +2.21 pp | 47.12% | 0.239 |
| +internationalism | 31.47% | 40.51% | +1.75 pp | 49.20% | 0.237 |
| +prefixes | 32.28% | 40.89% | +0.38 pp | 49.94% | 0.236 |
| +depleophony | 32.27% | 41.07% | +0.18 pp | 50.14% | 0.235 |
| +nasals | 32.64% | 41.17% | +0.09 pp | 50.24% | 0.235 |
| +proto-derived | 35.36% | 42.69% | +1.52 pp | 52.80% | 0.232 |
| +intl-preference | 35.44% | 42.77% | +0.09 pp | 52.84% | 0.231 |
| +adj-fleeting | 36.67% | 44.42% | +1.64 pp | 54.64% | 0.229 |
| +synonym-alts | 36.67% | 44.42% | +0.00 pp | 54.80% | 0.229 |
| +prefix-strip | 37.03% | 44.48% | +0.07 pp | 55.02% | 0.229 |
| +loan-stem-repair | 38.48% | 45.98% | +1.49 pp | 56.50% | 0.226 |
| +explicit-etymology (production) | 39.12% | 46.29% | +0.32 pp | 57.13% | 0.228 |

- **baseline** — Transliterate the first available form; no branch balancing, no repairs (the original prototype behavior).
- **+branch-consensus** — Branch-balanced skeleton vote + South-Slavic representative.
- **+six-subgroup** — Six dialect-subgroup vote with population tie-break (§4.1).
- **+lemma-endings** — Native POS lemma endings: noun nom.sg, adj -y/-i, verb -ti (§3).
- **+internationalism** — Internationalism ending table: -izm/-cija/-ičny/-alny/-ovati (§5.2).
- **+prefixes** — Normalize verbal/nominal prefixes råz-/prěd- (§2).
- **+depleophony** — Undo East-Slavic pleophony / liquid metathesis (§2).
- **+nasals** — Recover ę/ų nasal vowels from Polish (§2 Phase C).
- **+proto-derived** — Two-stage §4.4: consensus picks the root, the Proto-Slavic rule engine supplies the flavored form (ě/ć/đ/å/ȯ/y) via a leakage-free descendant+gloss link. Requires the proto cache.
- **+intl-preference** — Prefer the internationalism cluster over native synonyms (ISV design criteria favor international roots for modern vocabulary): aeroplan over samolot.
- **+adj-fleeting** — Drop a South-Slavic adjective's fleeting vowel before -y, gated on East/West consonant adjacency (dobar→dobry, zelen stays).
- **+synonym-alts** — Seed alternatives from secondary translations (below every primary candidate) so the official lemma surfaces in top-3/top-5 when it is a 2nd/3rd translation.
- **+prefix-strip** — Grow proto-link coverage: strip a shared prefix off the cognates, link the bare root, re-attach the Interslavic prefix (råzprostirati from *prostirati).
- **+loan-stem-repair** — Repair national adaptation quirks the representative leaks into a loan stem: Polish y→i, South-Slavic epenthetic vowel (akcenat→akcent), -ac→-ec, final -ia→-ija, masculine -a drop — each corroborated by a cognate or the internationalism gate.
- **+explicit-etymology (production)** — Use Wiktionary's stated (lang→ancestor) etymology to pick the Proto-Slavic reconstruction directly, before the fuzzy descendant+gloss link — the precise ancestor the corpus site uses.

## Rejected rules — tested and reverted

Each is the production config plus one experimental rule. All regress accuracy on the benchmark and are therefore **not** in the production config, per the keep-only-if-it-improves rule.

| Experiment | exact top-1 | Δ exact | norm top-1 | Δ norm |
|---|---:|---:|---:|---:|
| prod+palatals | 38.83% | -0.28 pp | 45.88% | -0.42 pp |
| prod+jat | 38.25% | -0.87 pp | 46.29% | -0.01 pp |
| prod+adj-longform | 36.26% | -2.86 pp | 42.67% | -3.62 pp |
| prod+y-recovery | 36.33% | -2.79 pp | 42.77% | -3.52 pp |

- **prod+palatals** — Recover ć/đ (*tj/*dj) from South Slavic — modern reflexes are too noisy; derive from Proto-Slavic instead.
- **prod+jat** — Reconstruct jat ě from the cross-branch reflex — unreliable from modern reflexes.
- **prod+adj-longform** — Long-form (ru/pl/cs) adjective representative — East/West orthographic quirks outweigh the fleeting-vowel fix.
- **prod+y-recovery** — Recover *y from East/West where South merged *y→i — too aggressive, flips correct i→y.

## POS-specific accuracy (final config)

| POS | n | exact | normalized |
|---|---:|---:|---:|
| adj | 2896 | 34.56% | 43.99% |
| adv | 657 | 19.94% | 29.22% |
| noun | 8362 | 46.88% | 53.29% |
| num | 112 | 11.61% | 24.11% |
| pron | 99 | 37.37% | 38.38% |
| verb | 4174 | 30.52% | 37.35% |

## Branch coverage vs accuracy (final config)

| branches with the consensus form | n | normalized |
|---:|---:|---:|
| 0 | 48 | 47.92% |
| 1 | 3560 | 18.06% |
| 2 | 5591 | 39.58% |
| 3 | 7101 | 65.72% |

## Confidence calibration (final config)

High-confidence candidates should match the official dictionary more often than low-confidence ones.

| confidence | n | normalized match |
|---|---:|---:|
| high | 6836 | 67.66% |
| medium | 7238 | 36.79% |
| low | 2226 | 11.59% |

## Before / after

- Baseline normalized top-1: **35.23%**
- Final normalized top-1: **46.29%** (+11.06 pp)
- Baseline exact top-1: **27.52%**
- Final exact top-1: **39.12%** (+11.60 pp)

## Remaining systematic errors (final config)

Of **8754** misses, **2225** (25%) are near-misses (normalized edit < 0.20 — an ending/one-letter fix) and **6529** are farther (usually a different root chosen by Interslavic).

| Error class | count | share of misses |
|---|---:|---:|
| different root / derivation | 4455 | 50.9% |
| extra letter (epenthesis / ending) | 1612 | 18.4% |
| missing letter (fleeting vowel / cluster) | 1078 | 12.3% |
| single-letter substitution | 1065 | 12.2% |
| y / i distinction | 494 | 5.6% |
| flavored letter (ě/ę/ų/å/ć/đ) not recovered | 50 | 0.6% |

## Next recommended linguistic rules

The Proto-Slavic-derived-form path (§4.4) is implemented — consensus picks the root and the Proto-Slavic rule engine supplies the flavored form via a leakage-free descendant+gloss link. Yer resolution now uses a genuine **tense-yer rule** (yer before *j → i/y) plus **reflex-guided vocalization** (a lexically-ambiguous weak yer is retained when the reflexes vote to keep it: `*pьsati`→`pisati` vs `*bьrati`→`brati`), and a length-free **reflex-shape-agreement** ranking rule replaced the earlier length heuristic. Ranked next steps, from the remaining-error analysis:

1. **Expand Proto-Slavic link coverage.** Only meanings with a matched `sla-pro` reconstruction get the flavored derivation; raising cache coverage and loosening the link gate (without admitting bad links) directly grows the proto-derived slice.
2. **Reduce the reconstruction's non-yer errors** (endings, palatalizations) so the proto form can be trusted even when it disagrees with the reflexes — currently such disagreements defer to the reflexes, capping the proto gain.
3. **Divergent-root modeling (semantic families, §4.2 step 3).** The ~6529 far-misses are mostly cases where Interslavic picked a different root than the plurality skeleton; scoring candidate *roots* (not surface forms) over the six subgroups, clustered by the proto descendant graph, would recover many.
4. **Secondary-imperfective verb stems** (`-yva-/-iva-/-ava-`) and the agentive `-telj`/abstract `-teljstvo` suffixes, seen repeatedly in the verb/noun error tail.
5. **POS-specific gender/animacy inference** to pick the right nominal ending where the modern citation forms disagree.
