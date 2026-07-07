# Derivation benchmark (derive-eval)

**Denominator:** 2115 derivationally related official lemma pairs, mined by inverse suffix lookup over the official dictionary (18458 entries). **Leakage story:** the layer receives the official *base* lemma + POS and must produce the official *derivative* forward; it never sees the derivative. Pair *selection* shares alternation knowledge with the layer (a disclosed bias — pairs the miner cannot align are excluded), but forward generation must still choose the right suffix allomorph, seam alternation and flavored spelling. A small share of mined pairs are string coincidences rather than true derivations (e.g. vino→vinny 'wine→guilty'); they inflate both layers symmetrically and are counted in the disclosed selection bias. **Dev/holdout (seeded id split):** normalized 99.68% / 99.82% (559 held out).

| Metric | seam-aware layer | naive concat baseline | Δ |
|---|---:|---:|---:|
| exact | **96.03%** | 47.85% | +48.18pp |
| normalized | **99.72%** | 83.59% | +16.12pp |

## Per pattern

| pattern | pairs | exact | normalized | naive exact | naive normalized |
|---|---:|---:|---:|---:|---:|
| adv | 360 | 99.72% | 100.00% | 97.22% | 97.50% |
| dimka | 24 | 91.67% | 100.00% | 58.33% | 66.67% |
| ica | 13 | 92.31% | 100.00% | 84.62% | 92.31% |
| ne | 158 | 99.37% | 100.00% | 99.37% | 100.00% |
| ny | 446 | 86.10% | 99.10% | 59.64% | 72.65% |
| ost | 414 | 99.52% | 100.00% | 0.00% | 100.00% |
| sky | 145 | 91.72% | 98.62% | 76.55% | 83.45% |
| telj | 88 | 100.00% | 100.00% | 100.00% | 100.00% |
| teljka | 6 | 100.00% | 100.00% | 100.00% | 100.00% |
| teljstvo | 9 | 100.00% | 100.00% | 100.00% | 100.00% |
| vnoun | 452 | 99.34% | 100.00% | 0.00% | 59.51% |

## Nearest misses (dev split only — holdout misses are never published)

```
pattern,base,official,derived,naive
sky,dětę,dětsky,dětęsky,dětęsky
sky,frank,franksky,frančsky,franksky
ny,konopja,konopjany,konopjny,konopjny
ny,vŕh,vŕhny,vŕšny,vŕhny
ny,zemja,zemjany,zemjny,zemjny
```
