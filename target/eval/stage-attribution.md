# Stage-attribution histogram (V7 §2.3)

For each of the **8624** normalized misses (of 16300 benchmarkable meanings), the last pipeline stage whose output still folded to the official form — i.e. the stage that destroyed, or never produced, the correct answer. Computed by replaying the winning candidate's `RuleStep` trace.

| Stage | misses | share |
|---|---:|---:|
| 3-cluster/vote | 2698 | 31.3% |
| 8-merge-rank | 1792 | 20.8% |
| 0-root-absent | 1779 | 20.6% |
| 1-normalize/representative | 1563 | 18.1% |
| 7-endings | 636 | 7.4% |
| 6-proto-rule | 139 | 1.6% |
| 4-repair | 17 | 0.2% |

## Top causes within each stage

| Stage | detail | misses |
|---|---|---:|
| 3-cluster/vote | wrong-cluster | 2698 |
| 0-root-absent | evidence-gap | 1779 |
| 8-merge-rank | diff-root-editorial | 1581 |
| 1-normalize/representative | residual:length | 687 |
| 7-endings | ending-residual | 631 |
| 1-normalize/representative | residual:y/i | 457 |
| 1-normalize/representative | residual:substitution | 383 |
| 8-merge-rank | same-root-surface | 211 |
| 6-proto-rule | yers | 83 |
| 1-normalize/representative | residual:flavored-letter | 34 |
| 6-proto-rule | proto-rule-residual | 23 |
| 6-proto-rule | endings | 15 |
| 4-repair | loan-epenthesis | 6 |
| 4-repair | loan-ok-suffix | 4 |
| 6-proto-rule | liquid-metathesis | 4 |
| 6-proto-rule | syllabic-liquid | 4 |
| 7-endings | adj-hard-y | 4 |
| 4-repair | loan-y-i | 3 |
| 4-repair | nasal-vowel | 3 |
| 6-proto-rule | nasal-vowels | 3 |
| 6-proto-rule | prothesis | 3 |
| 6-proto-rule | soft-consonants | 3 |
| 1-normalize/representative | pick-representative | 2 |
| 4-repair | verb-stative-eti | 1 |
| 6-proto-rule | collective-je | 1 |
| 7-endings | noun-ost | 1 |
