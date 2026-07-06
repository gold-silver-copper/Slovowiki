# Oracle ladder (V7 §2.4) — DIAGNOSTIC ONLY

Each row makes ONE pipeline stage perfect (by reading the official answer) while everything downstream stays the real production engine, over **16300** benchmarkable meanings. This path can never feed production; it exists only to rank stages by recoverable headroom. Spend effort top-down by Δ exact.

| Stage oracle | exact top-1 | Δ exact | norm top-1 | Δ norm |
|---|---:|---:|---:|---:|
| baseline (production) | 39.92% | — | 47.09% | — |
| oracle-cluster | 43.83% | +3.91pp | 53.20% | +6.11pp |
| oracle-representative | 43.59% | +3.67pp | 52.69% | +5.60pp |
| oracle-proto-link | 42.47% | +2.55pp | 50.47% | +3.37pp |
| oracle-all | 50.60% | +10.68pp | 63.34% | +16.25pp |

- **oracle-cluster** — force the vote to the cluster whose consonant key matches the official lemma; representative + repairs then run on the right cluster.
- **oracle-representative** — pick the winning group's member whose folded form is closest to the official lemma.
- **oracle-proto-link** — link the reconstruction whose derived form is closest to the official lemma (linker upper bound).
- **oracle-all** — all three at once (an approximate ceiling for the stages below word-selection).
