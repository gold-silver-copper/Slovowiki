# Aspect-pair benchmark (aspect-eval)

**Frozen reproducible inventory:** 1440 deterministic 1:1 same-gloss, morphologically-related official ipf↔pf pairs (ordered manifest `aspect-pairs.tsv`, FNV-1a-64 `5ab3e19ec5d758dd`). **Scored denominator:** 1434 regular pairs; 6 closed suppletive pairs are production grammar but excluded from accuracy scoring so their held-out partner answers cannot leak. **Keep metrics:** both-correct (primary), either-correct, and consonant-root fingerprint consistency. **Leakage:** official aspect/gloss/root spelling selects the evaluation slice only; both baseline forms are independently generated from cognate cells, and pair repair sees only those generated forms plus their scores. The shared seeded hash holds out 431 scored pairs.

| model | n | both correct | either correct | fingerprint consistency |
|---|---:|---:|---:|---:|
| independent baseline | 1434 | 16.67% | 49.44% | 78.24% |
| +core suffix repair | 1434 | 17.50% | 48.68% | 89.75% |
| +prefix perfectivization (production) | 1434 | 17.92% | 48.61% | 90.86% |
| +secondary imperfectives and -ovati→-ovyvati (experimental; holdout-flat) | 1434 | 17.99% | 48.47% | 92.47% |

The secondary `-yva-/-iva-/-ava-` and `-ovati→-ovyvati` families are controlled by `AspectConfig.secondary_imperfectives`. They remain implemented but disabled in production because the rung is flat on holdout both-correct. The production prefix repair improves the declared primary **both-correct** metric with no breaks and improves consonant-root fingerprint consistency (131 pairs remain unrepaired), but it lowers the secondary either-correct metric. The `-ovati→-uje` present stem is exported and unit-tested grammar metadata, not part of this infinitive-pair accuracy metric; the paired table below discloses that tradeoff rather than relabeling it as a universal accuracy gain.


## Dev / holdout

| model / split | n | both correct | either correct | fingerprint consistency |
|---|---:|---:|---:|---:|
| baseline dev | 1003 | 17.15% | 50.05% | 77.97% |
| baseline holdout | 431 | 15.55% | 48.03% | 78.89% |
| suffix rung dev | 1003 | 18.05% | 49.35% | 89.63% |
| suffix rung holdout | 431 | 16.24% | 47.10% | 90.02% |
| prefix rung dev | 1003 | 18.44% | 49.25% | 90.73% |
| prefix rung holdout | 431 | 16.71% | 47.10% | 91.18% |
| secondary experimental dev | 1003 | 18.54% | 49.05% | 92.22% |
| secondary experimental holdout | 431 | 16.71% | 47.10% | 93.04% |
| production dev | 1003 | 18.44% | 49.25% | 90.73% |
| production holdout | 431 | 16.71% | 47.10% | 91.18% |

## Paired significance vs independent baseline

| metric | fixed | broke | two-sided sign-test p |
|---|---:|---:|---:|
| both correct | 18 | 0 | 0.0000 |
| either correct | 3 | 15 | 0.0075 |

## Rule census

- `independent-roots-agree`: 1122
- `ipf-ati-to-pf-nuti`: 59
- `ipf-jati-to-pf-iti`: 2
- `pf-iti-to-ipf-jati`: 88
- `pf-nuti-to-ipf-ati`: 2
- `prefix-perfectivization`: 30
- `unrepaired`: 131

## Changed-pair sample

- mamiti ↔ omamiti: mamiti / obmanuti → mamiti / obmamiti (prefix-perfectivization)
- viti ↔ sviti: viti / aplesci → viti / aplesci (unrepaired)
- naduživati ↔ nadužiti: zneuživati / zlupotrebiti → zlupotrebjati / zlupotrebiti (pf-iti-to-ipf-jati)
- zloupotrěbjati ↔ zloupotrěbiti: zneuživati / zlupotrebiti → zlupotrebjati / zlupotrebiti (pf-iti-to-ipf-jati)
- sȯvŕšati ↔ sȯvŕšiti: dovršovati / soveršiti → soveršjati / soveršiti (pf-iti-to-ipf-jati)
- nastavjati ↔ nastaviti: regulavac / nastaviti → regulavac / nastaviti (unrepaired)
- dopušćati ↔ dopustiti: udelovati / dapuscic → udelovati / dapuscic (unrepaired)
- odrađati ↔ odraditi: otgovarivati / odraditi → odradjati / odraditi (pf-iti-to-ipf-jati)
- odčuđati ↔ odčuđiti: otčuždati / otdeliti → otčuždati / otčuždnųti (ipf-ati-to-pf-nuti)
- sŕditi ↔ råzsŕditi: sŕditi / råzrditi → sŕditi / råzsŕditi (prefix-perfectivization)
- obvěšćati ↔ obvěstiti: oznamovati / obhaneti → oznamovati / oznamovnųti (ipf-ati-to-pf-nuti)
- prědstavati ↔ prědstati: postaviti sę / stati → postaviti sę / stati (unrepaired)
- odobrjati ↔ odobriti: shvalovati / odobriti → odobrjati / odobriti (pf-iti-to-ipf-jati)
- uręđati ↔ uręditi: uporadočivati / usporadati → uporadočivati / usporadati (unrepaired)
- pytati ↔ spytati: pytati / spytac → pytati / spytati (prefix-perfectivization)
- sprašati ↔ sprositi: sprašivati / požadati → sprašivati / sprašivnųti (ipf-ati-to-pf-nuti)
- zapytyvati ↔ zapytati: pytati / zapytac → pytati / zapytati (prefix-perfectivization)
- uvěrjati ↔ uvěriti: ujištovati / uveriti → uverjati / uveriti (pf-iti-to-ipf-jati)
- ověrjati ↔ ověriti: overovati / podtverditi → podtverdjati / podtverditi (pf-iti-to-ipf-jati)
- upȯlnomoćevati ↔ upȯlnomoćiti: zplnomonjovati / upolnomočiti → zplnomonjovati / zplnomonjovnųti (ipf-ati-to-pf-nuti)
- izběgati ↔ izběgti: unikac / vyhnout se → unikac / vyhnout se (unrepaired)
- bajati ↔ nabajati: bajiti / naboltati → bajiti / naboltati (unrepaired)
- vȯzkresati ↔ vȯzkresnųti: voskresati / vȯzkrisnųti → vȯzkrisati / vȯzkrisnųti (pf-nuti-to-ipf-ati)
- odbivati ↔ odbiti: odražati / adbic → odražati / adbic (unrepaired)
- vȯzrastati ↔ vȯzråsti: odrastati / vyrasti → odrastati / vyrasti (unrepaired)
- mlåděti ↔ omlåděti: mladnouti / pomladěti → mladnouti / pomladěti (unrepaired)
- umoljati ↔ umoliti: prositi / umoliti → prositi / uprositi (prefix-perfectivization)
- načinati ↔ načęti: počinjati / začęti → počinjati / začęti (unrepaired)
- začinati ↔ začęti: počinjati / začęti → počinjati / začęti (unrepaired)
- ostavjati ↔ ostaviti: zaveštavati / odkazati → zaveštavati / zaveštavnųti (ipf-ati-to-pf-nuti)
- urěkati ↔ urěkti: zaklinati / ureći → zaklinati / zaklinnųti (ipf-ati-to-pf-nuti)
- odkųšati ↔ odkųsiti: odgrizati / otkusiti → odgrizati / odgriznųti (ipf-ati-to-pf-nuti)
- pozajmati ↔ pozajęti: zaimstvovati / požityčiti → požityčjati / požityčiti (pf-iti-to-ipf-jati)
- prědavati ↔ prědati: vysilati / prědati → vysilati / prědati (unrepaired)
- budovati ↔ izbudovati: stavjati / vibudovati → stavjati / staviti (ipf-jati-to-pf-iti)
- obrěmenjati ↔ obrěmeniti: zatežkavati / obremeniti → zatežkavati / zatežkavnųti (ipf-ati-to-pf-nuti)
- obtęžati ↔ obtęžiti: obtažovati / obremeniti → obtažovati / obtažovnųti (ipf-ati-to-pf-nuti)
- žegti ↔ izgorěti: paliti / izgoreti → paliti / izgoreti (unrepaired)
- prskati ↔ prsknųti: praskati / lopnuti → praskati / prasknųti (ipf-ati-to-pf-nuti)
- kalkulovati ↔ izkalkulovati: kalkulavac / viličic → kalkulavac / viličic (unrepaired)
