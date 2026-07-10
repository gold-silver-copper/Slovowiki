# Evaluation methodology — statistical instruments

## Overfitting guard — seeded 75/25 dev/holdout split

Entries are split by a deterministic hash of their dictionary id (~25% held out, **4078** entries; the split never changes). Rules are developed against dev; a kept rule must not gain on dev while flat/negative on holdout — that gap is the overfitting signal. The dev−holdout gap for the production config should stay within the holdout's sampling noise (±~1pp).

| Rung | exact dev | exact holdout | gap | norm dev | norm holdout | gap |
|---|---:|---:|---:|---:|---:|---:|
| baseline | 27.91% | 26.34% | +1.57 | 35.64% | 34.01% | +1.63 |
| +branch-consensus | 28.47% | 26.83% | +1.65 | 36.79% | 34.75% | +2.04 |
| +six-subgroup | 28.70% | 27.10% | +1.61 | 37.05% | 35.04% | +2.01 |
| +lemma-endings | 30.78% | 28.96% | +1.82 | 39.60% | 37.22% | +2.38 |
| +internationalism | 32.07% | 30.31% | +1.76 | 41.33% | 38.79% | +2.53 |
| +prefixes | 32.88% | 31.19% | +1.68 | 41.70% | 39.21% | +2.48 |
| +depleophony | 32.87% | 31.17% | +1.70 | 41.88% | 39.41% | +2.47 |
| +nasals | 33.24% | 31.56% | +1.68 | 41.97% | 39.50% | +2.46 |
| +proto-derived | 36.76% | 34.99% | +1.77 | 44.10% | 42.01% | +2.09 |
| +intl-preference | 36.84% | 35.09% | +1.75 | 44.18% | 42.10% | +2.08 |
| +adj-fleeting | 37.92% | 36.54% | +1.38 | 45.69% | 43.87% | +1.82 |
| +synonym-alts | 37.92% | 36.54% | +1.38 | 45.69% | 43.87% | +1.82 |
| +prefix-strip | 38.41% | 37.20% | +1.21 | 45.87% | 44.09% | +1.78 |
| +loan-stem-repair | 39.83% | 38.67% | +1.16 | 47.32% | 45.59% | +1.74 |
| +verb-class | 39.89% | 38.67% | +1.22 | 47.40% | 45.61% | +1.79 |
| +voicing | 39.97% | 38.72% | +1.25 | 47.50% | 45.66% | +1.84 |
| +explicit-etymology | 40.19% | 39.14% | +1.05 | 47.47% | 45.98% | +1.49 |
| +medoid-rep | 41.26% | 40.29% | +0.97 | 49.21% | 47.94% | +1.27 |
| +deriv-suffixes | 41.51% | 40.53% | +0.97 | 49.35% | 48.11% | +1.23 |
| +loan-hiatus | 41.58% | 40.56% | +1.02 | 49.43% | 48.14% | +1.29 |
| +spirantization (production) | 41.90% | 40.93% | +0.97 | 49.90% | 48.68% | +1.23 |

## Ladder-rung significance (paired sign test)

Each rung vs the previous rung, paired per entry: `fixed` = newly matched, `broke` = newly missed, on the **exact** metric (the primary keep-metric) and the normalized metric. p is the two-sided sign test on the discordant pairs — a rung whose p ≳ 0.05 on its keep-metric is not distinguishable from noise on this benchmark and should be treated as provisional, not proven.

| Rung | Δ exact | fixed/broke (exact) | p (exact) | Δ norm | fixed/broke (norm) | p (norm) |
|---|---:|---:|---:|---:|---:|---:|
| +branch-consensus | +0.55pp | 1077/988 | 0.0528 | +1.04pp | 1595/1425 | 0.0021 |
| +six-subgroup | +0.24pp | 138/99 | 0.0136 | +0.27pp | 198/154 | 0.0219 |
| +lemma-endings | +2.02pp | 335/5 | 0.0000 | +2.46pp | 410/9 | 0.0000 |
| +internationalism | +1.31pp | 217/4 | 0.0000 | +1.69pp | 279/4 | 0.0000 |
| +prefixes | +0.82pp | 134/0 | 0.0000 | +0.38pp | 62/0 | 0.0000 |
| +depleophony | -0.01pp | 0/2 | 0.4795 | +0.18pp | 39/9 | 0.0000 |
| +nasals | +0.37pp | 62/1 | 0.0000 | +0.09pp | 23/8 | 0.0119 |
| +proto-derived | +3.50pp | 693/122 | 0.0000 | +2.23pp | 494/131 | 0.0000 |
| +intl-preference | +0.09pp | 28/14 | 0.0449 | +0.09pp | 28/14 | 0.0449 |
| +adj-fleeting | +1.17pp | 196/6 | 0.0000 | +1.57pp | 263/7 | 0.0000 |
| +synonym-alts | +0.00pp | 0/0 | 1.0000 | +0.00pp | 0/0 | 1.0000 |
| +prefix-strip | +0.54pp | 96/8 | 0.0000 | +0.19pp | 39/8 | 0.0000 |
| +loan-stem-repair | +1.43pp | 244/11 | 0.0000 | +1.47pp | 253/14 | 0.0000 |
| +verb-class | +0.04pp | 11/4 | 0.1213 | +0.06pp | 14/4 | 0.0339 |
| +voicing | +0.07pp | 13/1 | 0.0033 | +0.09pp | 15/1 | 0.0012 |
| +explicit-etymology | +0.27pp | 204/160 | 0.0242 | +0.06pp | 215/205 | 0.6605 |
| +medoid-rep | +1.09pp | 550/372 | 0.0000 | +1.79pp | 821/529 | 0.0000 |
| +deriv-suffixes | +0.25pp | 40/0 | 0.0000 | +0.15pp | 24/0 | 0.0000 |
| +loan-hiatus | +0.06pp | 10/0 | 0.0044 | +0.07pp | 11/0 | 0.0026 |
| +spirantization (production) | +0.33pp | 57/3 | 0.0000 | +0.49pp | 83/3 | 0.0000 |

## Headline uncertainty (percentile bootstrap, 1000 seeded resamples)

- exact top-1 **41.66%** (95% CI 40.91–42.38%)
- normalized top-1 **49.60%** (95% CI 48.84–50.33%)

Deltas smaller than ~half this interval width should not be read as real without the paired test above (the paired test is far more sensitive than comparing two independent CIs).

## Score calibration (production top-1 score as P(normalized match))

| score bin | n | mean score | empirical match | gap |
|---|---:|---:|---:|---:|
| 0.3–0.4 | 453 | 0.393 | 0.130 | -0.263 |
| 0.4–0.5 | 4213 | 0.447 | 0.192 | -0.256 |
| 0.5–0.6 | 2887 | 0.547 | 0.390 | -0.157 |
| 0.6–0.7 | 1521 | 0.650 | 0.583 | -0.067 |
| 0.7–0.8 | 1301 | 0.750 | 0.663 | -0.088 |
| 0.8–0.9 | 1266 | 0.846 | 0.743 | -0.103 |
| 0.9–1.0 | 4659 | 0.948 | 0.730 | -0.218 |

- **ECE (expected calibration error): 0.1847** — mean |score − empirical match rate| weighted by bin size; 0 is perfectly calibrated.
- **Brier score: 0.2324** (lower is better; a constant base-rate predictor scores 0.2500).
- The three-way confidence badge (high/medium/low, thresholds 0.72/0.45 in `Confidence::from_score`) is derived from this score; if a bin's gap drifts past ~0.1 the thresholds should be re-fit.

### Isotonic recalibration (fit on dev, validated on holdout)

A monotone score→probability map (decile histogram + pool-adjacent-violators) fit on the dev split only, then applied to the untouched holdout:

| Holdout metric | raw score | recalibrated | Δ |
|---|---:|---:|---:|
| ECE | 0.1947 | 0.0128 | -0.1819 |
| Brier | 0.2339 | 0.1953 | -0.0386 |

The recalibrated probability is what a downstream consumer (site reliability badge, novel-word filter) should read as *P(matches the official lemma)*; the raw score remains the ranking key. Refit whenever the ladder changes.

### Proposal-filter operating points (calibrated p, holdout split)

| threshold | n ≥ t | precision | recall |
|---:|---:|---:|---:|
| ≥ 0.3 | 2859 | 61.7% | 88.9% |
| ≥ 0.4 | 2212 | 68.9% | 76.8% |
| ≥ 0.5 | 2212 | 68.9% | 76.8% |
| ≥ 0.6 | 1835 | 71.8% | 66.3% |
| ≥ 0.7 | 1490 | 72.9% | 54.7% |

The site's novel-word buckets (`export`) read these operating points: **propose** = calibrated p at the high-precision cutoff, **review** = the middle band, below = not shown. The committed calibrator is `data/score-calibration.json`.
