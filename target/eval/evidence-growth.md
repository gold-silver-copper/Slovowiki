# Evidence growth vs the root-absent ceiling (evidence-eval)

**Denominator:** 16300 benchmarkable meanings; the Wiktionary lemma cache holds 46654 lemmas. **Leakage story:** the cache never saw the `isv` answer; matching uses English gloss tokens + POS only; augmentation fills ONLY languages the dictionary row does not cite, so the dictionary's own evidence is never displaced.

| Measurement | value |
|---|---:|
| baseline root-absent misses | 1854 (11.4% of meanings) |
| recoverable from the cache (official root present under a gloss-matched lemma) | 63 (3.4% of root-absent) |
| root-absent after augmentation | 1854 (11.4%) |
| accuracy: baseline → augmented (exact) | 41.65% → 41.65% (+0.00pp) |
| accuracy: baseline → augmented (normalized) | 49.59% → 49.58% (-0.01pp) |
| paired sign test (normalized) | fixed 0 / broke 1, p = 1.0000 |

The native uk/sr/bg/sl Wiktionary enrichment named in issue #4 is **data-blocked** (no per-language wiktextract dumps on disk; enrichment affects display only, not benchmark evidence) and is recorded as out of scope here.
