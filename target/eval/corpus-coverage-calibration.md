# Corpus coverage calibration

- Score domain: `corpus-coverage-score-v1` (`coverage-languages-branches-v1`)
- Labels: `official-pos-semantic-proxy-v1`. A negative means only that no compatible official sense was found; it is not proof that a reconstruction is linguistically wrong.
- Split: `fnv1a-id-mod-4-holdout-v1`; isotonic/PAVA fit uses train rows only.
- Coverage means recall over holdout semantic positives.
- Inputs: `data/slavic-lemmas.cache.json` `c9611dd774f8a9a1caee14d53fbef0d4192301676bd4f066466bc92080ba283a`; `data/official-isv.csv` `5265761404d6bda07df55d1069d26350b2c107731c63913f827054d793d46cff`.

| split | rows | semantic positives |
|---|---:|---:|
| train | 20203 | 3352 |
| holdout | 6834 | 1126 |

| holdout metric | raw | calibrated |
|---|---:|---:|
| ECE | 0.237488 | 0.010588 |
| Brier | 0.164632 | 0.108036 |

| operating point | selected | hits | precision | coverage |
|---|---:|---:|---:|---:|
| proposal p≥0.6 | 405 | 282 | 0.696296 | 0.250444 |
| review p≥0.3 | 995 | 528 | 0.530653 | 0.468917 |
