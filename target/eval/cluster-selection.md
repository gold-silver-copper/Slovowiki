# Cluster-selection headroom (Measurement #2)

The wrong-cluster miss bucket is mostly the official dictionary choosing a different (editorial) root than our plurality vote. This forces the winning cluster by a **leakage-free** rule (except `oracle-cluster`, which reads the answer as the ceiling) and scores the real pipeline over 16300 meanings. `cluster-hit%` is the share of meanings whose official root is in the evidence where the rule's top candidate lands on that root.

| Rule | exact | Δ exact | norm | Δ norm | cluster-hit |
|---|---:|---:|---:|---:|---:|
| production | 41.65% | +0.00pp | 49.59% | +0.00pp | 69.2% |
| max-langs | 40.96% | -0.69pp | 48.77% | -0.82pp | 67.5% |
| max-branches | 41.12% | -0.53pp | 48.82% | -0.77pp | 67.8% |
| intl-first | 41.43% | -0.22pp | 49.28% | -0.31pp | 68.6% |
| oracle-cluster | 46.12% | +4.47pp | 56.29% | +6.70pp | 84.9% |

- **production** — the real branch-balanced six-subgroup vote (reference).
- **max-langs / max-branches** — force the cluster attested by the most distinct languages / branches (a raw recognizability proxy).
- **intl-first** — force any internationalism cluster (tests extending the genesis=I preference to every meaning).
- **oracle-cluster** — force the official cluster (upper bound; reads the answer).
