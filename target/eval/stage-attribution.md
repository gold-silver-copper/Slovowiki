# Stage-attribution histogram (V7 §2.3)

For each of the **8217** normalized misses (of 16300 benchmarkable meanings), the last pipeline stage whose output still folded to the official form — i.e. the stage that destroyed, or never produced, the correct answer. Computed by replaying the winning candidate's `RuleStep` trace.

| Stage | misses | share |
|---|---:|---:|
| 3-cluster/vote | 2734 | 33.3% |
| 8-merge-rank | 1825 | 22.2% |
| 0-root-absent | 1793 | 21.8% |
| 1-normalize/representative | 1200 | 14.6% |
| 7-endings | 501 | 6.1% |
| 6-proto-rule | 129 | 1.6% |
| 4-repair | 35 | 0.4% |

## Top causes within each stage

| Stage | detail | misses |
|---|---|---:|
| 3-cluster/vote | wrong-cluster | 2734 |
| 0-root-absent | evidence-gap | 1793 |
| 8-merge-rank | diff-root-editorial | 1669 |
| 1-normalize/representative | residual:length | 534 |
| 7-endings | ending-residual | 494 |
| 1-normalize/representative | residual:substitution | 377 |
| 1-normalize/representative | residual:y/i | 262 |
| 8-merge-rank | same-root-surface | 156 |
| 6-proto-rule | yers | 75 |
| 1-normalize/representative | residual:flavored-letter | 26 |
| 6-proto-rule | proto-rule-residual | 24 |
| 4-repair | liquid-metathesis | 16 |
| 6-proto-rule | endings | 15 |
| 4-repair | loan-epenthesis | 8 |
| 7-endings | adj-hard-y | 6 |
| 6-proto-rule | liquid-metathesis | 4 |
| 6-proto-rule | soft-consonants | 4 |
| 4-repair | loan-y-i | 3 |
| 4-repair | nasal-vowel | 3 |
| 4-repair | spirantization-hg | 3 |
| 6-proto-rule | prothesis | 3 |
| 6-proto-rule | syllabic-liquid | 3 |
| 1-normalize/representative | pick-representative | 1 |
| 4-repair | loan-fem-a | 1 |
| 4-repair | verb-stative-eti | 1 |
| 6-proto-rule | collective-je | 1 |
| 7-endings | noun-ost | 1 |
