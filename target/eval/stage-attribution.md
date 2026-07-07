# Stage-attribution histogram (V7 §2.3)

For each of the **8332** normalized misses (of 16300 benchmarkable meanings), the last pipeline stage whose output still folded to the official form — i.e. the stage that destroyed, or never produced, the correct answer. Computed by replaying the winning candidate's `RuleStep` trace.

| Stage | misses | share |
|---|---:|---:|
| 3-cluster/vote | 2780 | 33.4% |
| 0-root-absent | 1793 | 21.5% |
| 8-merge-rank | 1791 | 21.5% |
| 1-normalize/representative | 1288 | 15.5% |
| 7-endings | 518 | 6.2% |
| 6-proto-rule | 130 | 1.6% |
| 4-repair | 32 | 0.4% |

## Top causes within each stage

| Stage | detail | misses |
|---|---|---:|
| 3-cluster/vote | wrong-cluster | 2780 |
| 0-root-absent | evidence-gap | 1793 |
| 8-merge-rank | diff-root-editorial | 1629 |
| 1-normalize/representative | residual:length | 558 |
| 7-endings | ending-residual | 511 |
| 1-normalize/representative | residual:substitution | 436 |
| 1-normalize/representative | residual:y/i | 267 |
| 8-merge-rank | same-root-surface | 162 |
| 6-proto-rule | yers | 75 |
| 1-normalize/representative | residual:flavored-letter | 26 |
| 6-proto-rule | proto-rule-residual | 24 |
| 4-repair | liquid-metathesis | 16 |
| 6-proto-rule | endings | 16 |
| 4-repair | loan-epenthesis | 8 |
| 7-endings | adj-hard-y | 6 |
| 6-proto-rule | liquid-metathesis | 4 |
| 6-proto-rule | soft-consonants | 4 |
| 4-repair | loan-y-i | 3 |
| 4-repair | nasal-vowel | 3 |
| 6-proto-rule | prothesis | 3 |
| 6-proto-rule | syllabic-liquid | 3 |
| 1-normalize/representative | pick-representative | 1 |
| 4-repair | loan-fem-a | 1 |
| 4-repair | verb-stative-eti | 1 |
| 6-proto-rule | collective-je | 1 |
| 7-endings | noun-ost | 1 |
