# Candidate-generation benchmark

Benchmark: reconstruct the official Interslavic lemma from the modern Slavic cognates in the official dictionary, **without showing the generator the answer**. Evaluated on **16300** benchmarkable single-word entries. Every rule is kept only if it improved measured accuracy.

- **Metrics.** *exact*: identical to the official flavored lemma; *normalized*: identical after reducing both to the standard alphabet (§1.3); *skeleton*: identical after an ASCII fold; *top-3/5*: any of the first N candidates matches (normalized); *mean edit*: mean normalized Levenshtein distance to the official lemma.

## Kept rules — cumulative ablation ladder

Each rung adds exactly one rule to the previous, so its accuracy delta is attributable. The last rung is the kept **production** configuration.

| Rung | exact top-1 | norm top-1 | Δ norm | top-3 | mean edit |
|---|---:|---:|---:|---:|---:|
| baseline | 27.38% | 34.96% | +0.00 pp | 42.89% | 0.253 |
| +branch-consensus | 28.06% | 36.24% | +1.28 pp | 44.50% | 0.251 |
| +six-subgroup | 28.28% | 36.48% | +0.24 pp | 44.33% | 0.251 |
| +lemma-endings | 30.09% | 38.36% | +1.88 pp | 46.50% | 0.240 |
| +internationalism | 31.37% | 40.01% | +1.65 pp | 48.43% | 0.239 |
| +prefixes | 32.23% | 40.34% | +0.33 pp | 49.10% | 0.238 |
| +depleophony | 32.21% | 40.53% | +0.18 pp | 49.31% | 0.237 |
| +nasals | 32.42% | 40.43% | -0.10 pp | 49.22% | 0.238 |
| +proto-derived (production) | 34.15% | 40.77% | +0.34 pp | 51.17% | 0.238 |

- **baseline** — Transliterate the first available form; no branch balancing, no repairs (the original prototype behavior).
- **+branch-consensus** — Branch-balanced skeleton vote + South-Slavic representative.
- **+six-subgroup** — Six dialect-subgroup vote with population tie-break (§4.1).
- **+lemma-endings** — Native POS lemma endings: noun nom.sg, adj -y/-i, verb -ti (§3).
- **+internationalism** — Internationalism ending table: -izm/-cija/-ičny/-alny/-ovati (§5.2).
- **+prefixes** — Normalize verbal/nominal prefixes råz-/prěd- (§2).
- **+depleophony** — Undo East-Slavic pleophony / liquid metathesis (§2).
- **+nasals** — Recover ę/ų nasal vowels from Polish (§2 Phase C).
- **+proto-derived (production)** — Two-stage §4.4: consensus picks the root, the Proto-Slavic rule engine supplies the flavored form (ě/ć/đ/å/ȯ/y) via a leakage-free descendant+gloss link. Requires the proto cache.

## Rejected rules — tested and reverted

Each is the production config plus one experimental rule. All regress accuracy on the benchmark and are therefore **not** in the production config, per the keep-only-if-it-improves rule.

| Experiment | exact top-1 | Δ exact | norm top-1 | Δ norm |
|---|---:|---:|---:|---:|
| prod+palatals | 33.77% | -0.38 pp | 40.25% | -0.52 pp |
| prod+jat | 33.48% | -0.66 pp | 40.77% | +0.00 pp |
| prod+adj-longform | 31.87% | -2.28 pp | 38.01% | -2.77 pp |
| prod+y-recovery | 27.13% | -7.01 pp | 32.72% | -8.06 pp |

- **prod+palatals** — Recover ć/đ (*tj/*dj) from South Slavic — modern reflexes are too noisy; derive from Proto-Slavic instead.
- **prod+jat** — Reconstruct jat ě from the cross-branch reflex — unreliable from modern reflexes.
- **prod+adj-longform** — Long-form (ru/pl/cs) adjective representative — East/West orthographic quirks outweigh the fleeting-vowel fix.
- **prod+y-recovery** — Recover *y from East/West where South merged *y→i — too aggressive, flips correct i→y.

## POS-specific accuracy (final config)

| POS | n | exact | normalized |
|---|---:|---:|---:|
| adj | 2896 | 22.76% | 29.87% |
| adv | 657 | 18.87% | 28.16% |
| noun | 8362 | 42.86% | 48.54% |
| num | 112 | 8.93% | 22.32% |
| pron | 99 | 36.36% | 37.37% |
| verb | 4174 | 27.62% | 35.34% |

## Branch coverage vs accuracy (final config)

| branches with the consensus form | n | normalized |
|---:|---:|---:|
| 0 | 214 | 32.24% |
| 1 | 4458 | 22.66% |
| 2 | 6521 | 37.59% |
| 3 | 5107 | 61.01% |

## Confidence calibration (final config)

High-confidence candidates should match the official dictionary more often than low-confidence ones.

| confidence | n | normalized match |
|---|---:|---:|
| high | 5820 | 63.66% |
| medium | 8267 | 32.90% |
| low | 2213 | 9.99% |

## Before / after

- Baseline normalized top-1: **34.96%**
- Final normalized top-1: **40.77%** (+5.81 pp)
- Baseline exact top-1: **27.38%**
- Final exact top-1: **34.15%** (+6.77 pp)

## Remaining systematic errors (final config)

Of **9654** misses, **2783** (29%) are near-misses (normalized edit < 0.20 — an ending/one-letter fix) and **6871** are farther (usually a different root chosen by Interslavic).

| Error class | count | share of misses |
|---|---:|---:|
| different root / derivation | 4508 | 46.7% |
| extra letter (epenthesis / ending) | 2223 | 23.0% |
| single-letter substitution | 1177 | 12.2% |
| missing letter (fleeting vowel / cluster) | 1030 | 10.7% |
| y / i distinction | 654 | 6.8% |
| flavored letter (ě/ę/ų/å/ć/đ) not recovered | 62 | 0.6% |

## Next recommended linguistic rules

The Proto-Slavic-derived-form path (§4.4) is now implemented — consensus picks the root and the Proto-Slavic rule engine supplies the flavored form via a leakage-free descendant+gloss link — which is the source of the `+proto-derived` gain. Ranked next steps, from the remaining-error analysis:

1. **Expand Proto-Slavic link coverage.** Only meanings with a matched `sla-pro` reconstruction get the flavored derivation; raising cache coverage and loosening the link gate (without admitting bad links) directly grows the proto-derived slice.
2. **Tense-yer / Havlík refinement.** Pure Havlík over-drops yers that Interslavic vocalizes (`*pьsati`→`pisati`); the current POS-aware length guard is a heuristic. A proper tense-yer rule (yers before *j and in specific stems → i/y) would let the proto form win more verbs safely.
3. **Divergent-root modeling (semantic families, §4.2 step 3).** The ~6871 far-misses are mostly cases where Interslavic picked a different root than the plurality skeleton; scoring candidate *roots* (not surface forms) over the six subgroups, clustered by the proto descendant graph, would recover many.
4. **Secondary-imperfective verb stems** (`-yva-/-iva-/-ava-`) and the agentive `-telj`/abstract `-teljstvo` suffixes, seen repeatedly in the verb/noun error tail.
5. **POS-specific gender/animacy inference** to pick the right nominal ending where the modern citation forms disagree.
