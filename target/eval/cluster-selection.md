# Cluster-selection headroom (Measurement #2)

The wrong-cluster miss bucket is mostly the official dictionary choosing a different (editorial) root than our plurality vote. This forces the winning cluster by a **leakage-free** rule (except `oracle-cluster`, which reads the answer as the ceiling) and scores the real pipeline over 16300 meanings. `cluster-hit%` is the share of meanings whose official root is in the evidence where the rule's top candidate lands on that root.

| Rule | exact | Δ exact | norm | Δ norm | cluster-hit |
|---|---:|---:|---:|---:|---:|
| production | 39.92% | +0.00pp | 47.09% | +0.00pp | 70.1% |
| max-langs | 39.17% | -0.75pp | 46.12% | -0.97pp | 68.5% |
| max-branches | 39.39% | -0.53pp | 46.29% | -0.80pp | 68.9% |
| intl-first | 39.66% | -0.26pp | 46.74% | -0.35pp | 69.5% |
| oracle-cluster | 43.82% | +3.90pp | 53.20% | +6.10pp | 84.9% |

- **production** — the real branch-balanced six-subgroup vote (reference).
- **max-langs / max-branches** — force the cluster attested by the most distinct languages / branches (a raw recognizability proxy).
- **intl-first** — force any internationalism cluster (tests extending the genesis=I preference to every meaning).
- **oracle-cluster** — force the official cluster (upper bound; reads the answer).
