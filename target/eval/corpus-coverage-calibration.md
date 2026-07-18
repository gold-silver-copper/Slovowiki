# Corpus coverage calibration

- Score domain: `corpus-coverage-score-v1` (`coverage-languages-branches-v1`)
- Labels: `official-pos-semantic-proxy-byforms-v2`. A negative means only that no compatible official sense was found; it is not proof that a reconstruction is linguistically wrong.
- Split: `fnv1a-id-mod-4-holdout-v1`; isotonic/PAVA fit uses train rows only.
- Coverage means recall over holdout semantic positives.
- Inputs: `data/slavic-lemmas.cache.json` `c9611dd774f8a9a1caee14d53fbef0d4192301676bd4f066466bc92080ba283a`; `data/official-isv.csv` `5265761404d6bda07df55d1069d26350b2c107731c63913f827054d793d46cff`.

| split | rows | semantic positives |
|---|---:|---:|
| train | 20203 | 3405 |
| holdout | 6834 | 1140 |

| holdout metric | raw | calibrated |
|---|---:|---:|
| ECE | 0.235440 | 0.010651 |
| Brier | 0.164402 | 0.108828 |

| unfiltered holdout operating point (not proposal-list quality) | selected | hits | precision | coverage |
|---|---:|---:|---:|---:|
| proposal p≥0.6 | 405 | 284 | 0.701235 | 0.249123 |
| review p≥0.3 | 995 | 534 | 0.536683 | 0.468421 |
