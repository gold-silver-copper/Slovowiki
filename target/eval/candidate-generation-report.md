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
| +lemma-endings | 30.33% | 39.01% | +2.46 pp | 47.47% | 0.238 |
| +internationalism | 31.63% | 40.69% | +1.69 pp | 49.48% | 0.236 |
| +prefixes | 32.45% | 41.07% | +0.38 pp | 50.22% | 0.235 |
| +depleophony | 32.44% | 41.26% | +0.18 pp | 50.42% | 0.235 |
| +nasals | 32.82% | 41.35% | +0.09 pp | 50.52% | 0.235 |
| +proto-derived | 36.32% | 43.58% | +2.23 pp | 53.67% | 0.230 |
| +intl-preference | 36.40% | 43.66% | +0.09 pp | 53.71% | 0.229 |
| +adj-fleeting | 37.57% | 45.23% | +1.57 pp | 55.47% | 0.227 |
| +synonym-alts | 37.57% | 45.23% | +0.00 pp | 55.63% | 0.227 |
| +prefix-strip | 38.11% | 45.42% | +0.19 pp | 55.95% | 0.227 |
| +loan-stem-repair | 39.54% | 46.89% | +1.47 pp | 57.42% | 0.224 |
| +verb-class | 39.58% | 46.95% | +0.06 pp | 57.47% | 0.224 |
| +voicing | 39.66% | 47.04% | +0.09 pp | 57.58% | 0.224 |
| +explicit-etymology | 39.93% | 47.10% | +0.06 pp | 57.90% | 0.226 |
| +medoid-rep | 41.02% | 48.89% | +1.79 pp | 59.57% | 0.226 |
| +deriv-suffixes | 41.26% | 49.04% | +0.15 pp | 59.87% | 0.226 |
| +loan-hiatus | 41.33% | 49.10% | +0.07 pp | 59.94% | 0.226 |
| +spirantization (production) | 41.66% | 49.60% | +0.49 pp | 60.48% | 0.224 |

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
- **+verb-class** — Verb conjugation classes: jat after hushing spelled a (drzati, slysati), statives -eti on East/West e-stem evidence (kameneti).
- **+voicing** — Voicing correspondences: devoiced prefixes bes-/is- -> bez-/iz- and loan nz -> ns, each corroborated by a cognate with the voiced/Latin spelling.
- **+explicit-etymology** — Use Wiktionary's stated (lang→ancestor) etymology to pick the Proto-Slavic reconstruction directly, before the fuzzy descendant+gloss link — the precise ancestor the corpus site uses.
- **+medoid-rep** — Pick the winning cluster's representative as the medoid — the member minimizing total folded edit distance to the others (the most central attested form) — instead of the fixed REP_PRIORITY, avoiding dialectal/oblique outliers. Measured by rep-eval (+1.09pp exact), the biggest recoverable slice of the +3.7pp oracle-representative ceiling.
- **+deriv-suffixes** — Derivational-suffix normalization (root-consistency invariant [DERIV]), each categorical in the dictionary: -telj- kept before suffixes (53 -teljstvo/-teljny/-teljsky vs 0 hard), feminine i-stem soft -sť (516 vs 0), deverbal -livy (152 vs 0 -ljivy).
- **+loan-hiatus** — Keep the Graeco-Latin -ia-/-io- hiatus in internationalisms (socialny, entuziazm, sociolog) where the Slavic cognates' -ija- glide is a national adaptation: 24 -ial- vs 0 -ijal- in the dictionary, 139 midword -io- vs 1 -ijo-.
- **+spirantization (production)** — Undo the *g→h spirantization a Czech/Slovak/Ukrainian/Belarusian representative leaks into the surface (blahosklonnost→blago-), corroborated per consonant position by ≥2 g-preserving cognates (ru/pl/South). ISV has no g→h rule (RULE_SPEC §2); genuine *x/loan h stays because the g-preserving lects write h there too.

## Rejected rules — tested and reverted

Each is the production config plus one experimental rule. All regress accuracy on the benchmark and are therefore **not** in the production config, per the keep-only-if-it-improves rule.

| Experiment | exact top-1 | Δ exact | norm top-1 | Δ norm |
|---|---:|---:|---:|---:|
| prod+palatals | 41.31% | -0.34 pp | 49.14% | -0.45 pp |
| prod+jat | 40.95% | -0.71 pp | 49.59% | -0.01 pp |
| prod+adj-longform | 41.66% | +0.00 pp | 49.60% | +0.00 pp |
| prod+y-recovery | 38.92% | -2.74 pp | 45.97% | -3.63 pp |

- **prod+palatals** — Recover ć/đ (*tj/*dj) from South Slavic — modern reflexes are too noisy; derive from Proto-Slavic instead.
- **prod+jat** — Reconstruct jat ě from the cross-branch reflex — unreliable from modern reflexes.
- **prod+adj-longform** — Long-form (ru/pl/cs) adjective representative — East/West orthographic quirks outweigh the fleeting-vowel fix.
- **prod+y-recovery** — Recover *y from East/West where South merged *y→i — too aggressive, flips correct i→y.

## POS-specific accuracy (final config)

| POS | n | exact | normalized |
|---|---:|---:|---:|
| adj | 2896 | 37.22% | 48.93% |
| adv | 657 | 21.00% | 33.33% |
| noun | 8362 | 50.74% | 57.50% |
| num | 112 | 15.18% | 25.89% |
| pron | 99 | 41.41% | 44.44% |
| verb | 4174 | 30.50% | 37.54% |

## Branch coverage vs accuracy (final config)

| branches with the consensus form | n | normalized |
|---:|---:|---:|
| 0 | 13 | 23.08% |
| 1 | 3529 | 17.29% |
| 2 | 5558 | 41.65% |
| 3 | 7200 | 71.61% |

## Confidence calibration (final config)

High-confidence candidates should match the official dictionary more often than low-confidence ones.

| confidence | n | normalized match |
|---|---:|---:|
| high | 6989 | 72.21% |
| medium | 7096 | 39.05% |
| low | 2215 | 12.01% |

## Before / after

- Baseline normalized top-1: **35.23%**
- Final normalized top-1: **49.60%** (+14.36 pp)
- Baseline exact top-1: **27.52%**
- Final exact top-1: **41.66%** (+14.14 pp)

## Remaining systematic errors (final config)

Of **8216** misses, **1825** (22%) are near-misses (normalized edit < 0.20 — an ending/one-letter fix) and **6391** are farther (usually a different root chosen by Interslavic).

| Error class | count | share of misses |
|---|---:|---:|
| different root / derivation | 4480 | 54.5% |
| missing letter (fleeting vowel / cluster) | 1260 | 15.3% |
| extra letter (epenthesis / ending) | 1024 | 12.5% |
| single-letter substitution | 1011 | 12.3% |
| y / i distinction | 398 | 4.8% |
| flavored letter (ě/ę/ų/å/ć/đ) not recovered | 43 | 0.5% |

## Next recommended linguistic rules

The Proto-Slavic-derived-form path (§4.4) is implemented — consensus picks the root and the Proto-Slavic rule engine supplies the flavored form via a leakage-free descendant+gloss link. Yer resolution now uses a genuine **tense-yer rule** (yer before *j → i/y) plus **reflex-guided vocalization** (a lexically-ambiguous weak yer is retained when the reflexes vote to keep it: `*pьsati`→`pisati` vs `*bьrati`→`brati`), and a length-free **reflex-shape-agreement** ranking rule replaced the earlier length heuristic. Ranked next steps, from the remaining-error analysis:

1. **Expand Proto-Slavic link coverage.** Only meanings with a matched `sla-pro` reconstruction get the flavored derivation; raising cache coverage and loosening the link gate (without admitting bad links) directly grows the proto-derived slice.
2. **Reduce the reconstruction's non-yer errors** (endings, palatalizations) so the proto form can be trusted even when it disagrees with the reflexes — currently such disagreements defer to the reflexes, capping the proto gain.
3. **Divergent-root modeling (semantic families, §4.2 step 3).** The ~6391 far-misses are mostly cases where Interslavic picked a different root than the plurality skeleton; scoring candidate *roots* (not surface forms) over the six subgroups, clustered by the proto descendant graph, would recover many.
4. **Secondary-imperfective verb stems** (`-yva-/-iva-/-ava-`) and the agentive `-telj`/abstract `-teljstvo` suffixes, seen repeatedly in the verb/noun error tail.
5. **POS-specific gender/animacy inference** to pick the right nominal ending where the modern citation forms disagree.
