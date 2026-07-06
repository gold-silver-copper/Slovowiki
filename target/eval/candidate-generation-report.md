# Candidate-generation benchmark

Benchmark: reconstruct the official Interslavic lemma from the modern Slavic cognates in the official dictionary, **without showing the generator the answer**. Evaluated on **16300** benchmarkable single-word entries. Every rule is kept only if it improved measured accuracy.

- **Metrics.** *exact*: identical to the official flavored lemma; *normalized*: identical after reducing both to the standard alphabet (§1.3); *skeleton*: identical after an ASCII fold; *top-3/5*: any of the first N candidates matches (normalized); *mean edit*: mean normalized Levenshtein distance to the official lemma.

## Kept rules — cumulative ablation ladder

Each rung adds exactly one rule to the previous, so its accuracy delta is attributable. The last rung is the kept **production** configuration.

| Rung | exact top-1 | norm top-1 | Δ norm | top-3 | mean edit |
|---|---:|---:|---:|---:|---:|
| baseline | 27.53% | 35.26% | +0.00 pp | 43.29% | 0.252 |
| +branch-consensus | 28.06% | 36.28% | +1.02 pp | 44.68% | 0.251 |
| +six-subgroup | 28.30% | 36.55% | +0.27 pp | 44.47% | 0.250 |
| +lemma-endings | 30.10% | 38.77% | +2.22 pp | 47.13% | 0.238 |
| +internationalism | 31.44% | 40.48% | +1.71 pp | 49.17% | 0.237 |
| +prefixes | 32.30% | 40.83% | +0.36 pp | 49.89% | 0.236 |
| +depleophony | 32.29% | 41.02% | +0.18 pp | 50.09% | 0.235 |
| +nasals | 32.50% | 40.92% | -0.10 pp | 50.00% | 0.236 |
| +proto-derived | 34.90% | 42.09% | +1.17 pp | 52.27% | 0.233 |
| +intl-preference | 34.98% | 42.17% | +0.09 pp | 52.31% | 0.233 |
| +adj-fleeting | 36.21% | 43.83% | +1.66 pp | 54.29% | 0.230 |
| +synonym-alts | 36.21% | 43.83% | +0.00 pp | 54.46% | 0.230 |
| +prefix-strip (production) | 36.52% | 43.84% | +0.01 pp | 54.66% | 0.230 |

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
- **+prefix-strip (production)** — Grow proto-link coverage: strip a shared prefix off the cognates, link the bare root, re-attach the Interslavic prefix (råzprostirati from *prostirati).

## Rejected rules — tested and reverted

Each is the production config plus one experimental rule. All regress accuracy on the benchmark and are therefore **not** in the production config, per the keep-only-if-it-improves rule.

| Experiment | exact top-1 | Δ exact | norm top-1 | Δ norm |
|---|---:|---:|---:|---:|
| prod+palatals | 36.19% | -0.33 pp | 43.36% | -0.48 pp |
| prod+jat | 35.85% | -0.67 pp | 43.84% | +0.00 pp |
| prod+adj-longform | 33.06% | -3.47 pp | 39.47% | -4.37 pp |
| prod+y-recovery | 29.61% | -6.91 pp | 36.00% | -7.84 pp |

- **prod+palatals** — Recover ć/đ (*tj/*dj) from South Slavic — modern reflexes are too noisy; derive from Proto-Slavic instead.
- **prod+jat** — Reconstruct jat ě from the cross-branch reflex — unreliable from modern reflexes.
- **prod+adj-longform** — Long-form (ru/pl/cs) adjective representative — East/West orthographic quirks outweigh the fleeting-vowel fix.
- **prod+y-recovery** — Recover *y from East/West where South merged *y→i — too aggressive, flips correct i→y.

## POS-specific accuracy (final config)

| POS | n | exact | normalized |
|---|---:|---:|---:|
| adj | 2896 | 29.97% | 39.43% |
| adv | 657 | 19.63% | 29.07% |
| noun | 8362 | 44.02% | 50.54% |
| num | 112 | 9.82% | 23.21% |
| pron | 99 | 39.39% | 40.40% |
| verb | 4174 | 29.35% | 36.44% |

## Branch coverage vs accuracy (final config)

| branches with the consensus form | n | normalized |
|---:|---:|---:|
| 0 | 117 | 50.43% |
| 1 | 4497 | 24.35% |
| 2 | 6711 | 41.25% |
| 3 | 4975 | 64.80% |

## Confidence calibration (final config)

High-confidence candidates should match the official dictionary more often than low-confidence ones.

| confidence | n | normalized match |
|---|---:|---:|
| high | 5957 | 69.04% |
| medium | 8044 | 34.49% |
| low | 2299 | 11.27% |

## Before / after

- Baseline normalized top-1: **35.26%**
- Final normalized top-1: **43.84%** (+8.58 pp)
- Baseline exact top-1: **27.53%**
- Final exact top-1: **36.52%** (+8.99 pp)

## Remaining systematic errors (final config)

Of **9154** misses, **2555** (28%) are near-misses (normalized edit < 0.20 — an ending/one-letter fix) and **6599** are farther (usually a different root chosen by Interslavic).

| Error class | count | share of misses |
|---|---:|---:|
| different root / derivation | 4399 | 48.1% |
| extra letter (epenthesis / ending) | 1834 | 20.0% |
| single-letter substitution | 1180 | 12.9% |
| missing letter (fleeting vowel / cluster) | 1085 | 11.9% |
| y / i distinction | 601 | 6.6% |
| flavored letter (ě/ę/ų/å/ć/đ) not recovered | 55 | 0.6% |

## Next recommended linguistic rules

The Proto-Slavic-derived-form path (§4.4) is implemented — consensus picks the root and the Proto-Slavic rule engine supplies the flavored form via a leakage-free descendant+gloss link. Yer resolution now uses a genuine **tense-yer rule** (yer before *j → i/y) plus **reflex-guided vocalization** (a lexically-ambiguous weak yer is retained when the reflexes vote to keep it: `*pьsati`→`pisati` vs `*bьrati`→`brati`), and a length-free **reflex-shape-agreement** ranking rule replaced the earlier length heuristic. Ranked next steps, from the remaining-error analysis:

1. **Expand Proto-Slavic link coverage.** Only meanings with a matched `sla-pro` reconstruction get the flavored derivation; raising cache coverage and loosening the link gate (without admitting bad links) directly grows the proto-derived slice.
2. **Reduce the reconstruction's non-yer errors** (endings, palatalizations) so the proto form can be trusted even when it disagrees with the reflexes — currently such disagreements defer to the reflexes, capping the proto gain.
3. **Divergent-root modeling (semantic families, §4.2 step 3).** The ~6599 far-misses are mostly cases where Interslavic picked a different root than the plurality skeleton; scoring candidate *roots* (not surface forms) over the six subgroups, clustered by the proto descendant graph, would recover many.
4. **Secondary-imperfective verb stems** (`-yva-/-iva-/-ava-`) and the agentive `-telj`/abstract `-teljstvo` suffixes, seen repeatedly in the verb/noun error tail.
5. **POS-specific gender/animacy inference** to pick the right nominal ending where the modern citation forms disagree.
