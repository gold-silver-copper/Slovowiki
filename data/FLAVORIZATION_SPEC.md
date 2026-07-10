# Flavorization of Raw Source-Language Words → Interslavic Display Headwords

**A single implementable specification for a display-grade, per-language adapter.**

When Slovowiki shows a *raw* word attested in another Slavic language (the
306k-lemma `RawSlavicCorpus`, issues #33/#34), its display headword should be
that word **adapted to Interslavic orthography** — e.g. Polish `winyl` →
`vinyl`, Russian `пластинка` → `plastinka`, Macedonian `меѓу` → `medžu` — not
the verbatim national spelling. Today only Russian is transliterated
(`site.rs::source_display`); every Latin-script language passes through
verbatim, and the non-Russian Cyrillic languages (mk 39.6k, uk 25.1k, bg 15.5k,
be 4.7k, cu/orv/rue) render raw Cyrillic headwords.

**Terminology.** The ISV community uses *flavorizacija* (steen.free.fr
flavorizacija.html) for the reverse direction (rendering ISV text with a
national flavor). This spec defines the **ISV-ward** direction — adapting a
national form *into* ISV orthography — matching how the project already uses
the word. It is the display-grade sibling of the per-language `X_slo()`
normalizers in van Steenbergen's voting machine (see
`VOTING_MACHINE_NOTES.md`), which our `normalize.rs` partly reproduces for the
consensus vote.

**Source tags** (as in `RULE_SPEC.md`): `[ORTH]` interslavic.fun orthography,
`[PHON]` phonology, `[DESIGN]` design criteria, `[STEEN-G]` steen.free.fr
grammar cluster, `[STEEN-D]` steen.free.fr derivation/flavorizacija/loanwords,
`[VM]` the voting machine's per-language rules as recorded in
`data/VOTING_MACHINE_NOTES.md`, `[NORM]` the existing `src/normalize.rs`
voting normalizer.

---

## 0. Scope and non-scope

**In scope** — one pure function:

```
flavorize_word(lang: &str, pos: &str, word: &str) -> String
```

applied to *words displayed as words*: the raw-lemma display headword (and its
dedup fold, which is derived from it), cross-lingual "same meaning" chips,
cognate-member word mentions, and evidence-form displays.

**Out of scope:**

- **Running text** (etymology paragraphs, glosses, usage quotations): stays on
  plain script transliteration (`source_display` / `russian_translit`), because
  jat/ending adaptation of full sentences would misrepresent quoted material.
- **The consensus vote.** `normalize.rs` is tuned for cognate alignment and is
  benchmark-gated; the voting-machine port experiments (all REGRESSED, see
  `VOTING_MACHINE_NOTES.md` §"Ports tested") prove display-grade rules must
  not leak into it. `flavorize_word` is a separate module; `normalize.rs` is
  not touched.
- **Sound-change reversal that needs etymology** (Class C below): Belarusian
  akanne, Ukrainian ikavism, Czech/Slovak/Upper-Sorbian/uk/be *g→h
  restoration, Ekavian jat, Polish nasal-prothesis. These are only solvable
  with cognate evidence (the machinery in `consensus.rs` —
  `jat_reconstruction`, `nasal_from_polish`, `spirantize_h_to_g` — exists but
  requires a cognate set, which raw words by definition lack). Recorded in §8
  as explicit non-goals / future Layer-2 work.

---

## 1. Target orthography

Output alphabet = **ISV standard Latin** `[ORTH][STEEN-G]`:

```
a b c č d dž e ě f g h i j k l lj m n nj o p r s š t u v y z ž
```

plus capitals. `ě` is part of the standard alphabet and is deliberately
produced (see §3). **No other etymological letter may appear in output**: the
flavored letters `ę ų å ė ȯ ć đ ĺ ń ŕ t́ d́ ś ź` fold per `RULE_SPEC.md` §1.3
(`ę→e ų→u å→a ė→e ȯ→o ć→č đ→dž ĺ→l ń→n ŕ→r t́→t d́→d ś→s ź→z`) — a raw
attestation must not pretend to etymological precision the source spelling
does not carry.

Non-alphabetic characters (space, hyphen, apostrophe-as-punctuation, digits)
pass through. Combining stress/length marks (U+0300–U+036F) are stripped.

---

## 2. The algorithm

Five stages, deterministic, no lookup outside the word itself + its POS:

1. **Pre-pass.** Unicode NFC; strip combining marks U+0300–U+036F; record
   per-character case, then work on the lowercase form (case restored in
   stage 5, `russian_translit::push_capitalized` pattern; a digraph output
   from an uppercase source letter title-cases: `Щ→Šč`, `Џ→Dž`).
2. **Ending adaptation** (POS-gated, on the *source* spelling, longest match
   first — §2.2). Example: ru verb `-ть#` → `-ти` so stage 3 yields `-ti`.
3. **Per-language rewrite** (§4): an ordered list of context-sensitive rules
   (`longest-match-first`; contexts are word boundary `#`, "after consonant"
   `C_`, "before vowel" `_V`, evaluated on the source string). Cyrillic
   languages transliterate and adapt in the same pass.
4. **Common post-pass** (all languages): foreign-letter fold `w→v`, `x→ks`,
   `qu→kv`, `q→k`; any remaining Latin letter with a diacritic not in the
   target alphabet folds to its base letter.
5. **Validation + case restoration.** Every alphabetic output char must be in
   the §1 alphabet; any residue is kept verbatim but **counted and reported**
   (one loud stat line at export, listed in the coverage report — the PR #55
   "loud failure" philosophy; no silent garbage).

### 2.1 Rule classes

Every rule in §4 carries a class:

- **Class A — orthographic.** Pure spelling/script conversion, no phonological
  claim: `w→v`, `cz→č`, `ż→ž`, `х→h`, `ó→o`, Cyrillic base letters. Always
  safe. These are the rules the user-visible examples come from
  (`winyl→vinyl`).
- **Class B — regular correspondence.** A source grapheme whose ISV
  correspondent is deterministic *from spelling alone* for the large majority
  of native vocabulary, with a **known failure class** (usually loanwords)
  stated inline. Examples: cs `ů→o` (`kůň→konj`), sk `ä→e` (`mäso→meso`),
  mk `ќ→č`/`ѓ→dž` (`ноќ→noč`, `меѓу→medžu`), ru `ё→(j)e` (`самолёт→samolet`),
  Polish `ą→u`. Class B rules ship with per-rule golden tests; a rule whose
  failure class turns out to dominate gets demoted to Class C.
- **Class C — excluded** (needs etymology; §8). Never implemented in Layer 1.

### 2.2 Ending adaptation (Class B-morph)

Raw lemmas carry a POS (`RawSlavicLemma.pos`: noun/verb/adj/adv). Citation
forms differ mechanically from ISV citation forms; these are safe,
POS-gated, longest-match-first, and only at `#` (word end):

| POS | Languages | Rule | Example | Failure class |
|---|---|---|---|---|
| verb | ru, be | `-ть → -ти` (then stage 3 → `-ti`) | читать→čitati, быть→byti | `-чь` verbs excluded (ISV keeps `gt/kt`: mogti `[STEEN-G]`) |
| verb | uk, rue | already `-ти` — no-op | читати→čitati | — |
| verb | cs | `-t → -ti` | dělat→dělati, být→byti | `-ct` (moct) excluded |
| verb | sk | `-ť → -ti` | robiť→robiti | — |
| verb | pl, hsb, dsb, csb, szl | `-ć → -ti` | być→byti, pisać→pisati | `-c` (móc) excluded |
| verb | sl, sh/hr/bs | already `-ti`; sh `-ći` excluded | delati→delati | — |
| verb | bg, mk | **no rule** — lemma is a finite form (bg 1sg, mk 3sg), not an infinitive | чета stays četa | flagged in §8 |
| adj | ru | `-ый/-ий → -y` | русский→russky, синий→siny | — |
| adj | uk, be | `-ий/-і́й/-ы́й → -y` | добрий→dobry | — |
| adj | cs, sk | `-ý → -y` (long-vowel rule covers it); `-í` kept | nový→novy | soft-stem `-í` shown as `-i` |
| adj | sh/hr/bs, mk, bg | `-i → -y` (sh definite form) | novi→novy | mk/bg cited in indefinite — usually no-op |
| noun | all | none (nominative singular already aligns) | — | — |

Reflexive verb particles (ru `-ся`, pl `się`…) are **not** rewritten to ` sę`
in Layer 1 — the corpus's single-token gate makes them rare, and
`consensus.rs::strip_reflexive` shows the required care. Flagged in §9.

---

## 3. The soft-e / jat principle (the `ě` rule)

ISV `ě` is defined as the *palatalizing e* (`ʲɛ`, `RULE_SPEC.md` §1.2) and is
part of the standard alphabet. Several source orthographies mark "palatalized
e" explicitly; that marking maps to `ě` **uniformly**:

| Language | Source grapheme | Rule | Example (✓ = matches ISV lemma) |
|---|---|---|---|
| ru, be | `е` after consonant | → `ě` | дело→dělo ✓, река→rěka ✓, день→děnj (~ISV denj — see failure class) |
| ru, be | `е` at `#`, after vowel, after `ь/ъ` | → `je` | ель→jelj, объезд→objezd |
| uk | `є` after consonant | → `ě`; else → `je` | синє→syně; Євген→Jevhen |
| cs, hsb, dsb | `ě` | → `ě` (identity) | město→město ✓, dźěło→dělo ✓ |
| sk | `ie` (diphthong) | → `ě` | viera→věra ✓, biely→běly ✓ |
| sh/hr/bs (Ijekavian) | `ije` | → `ě` | rijeka→rěka ✓, lijep→lěp ✓ |
| sh/hr/bs (Ijekavian) | `Cje` (consonant + je) | → `Cě` | mjesto→město ✓, pjesma→pěsma (~ISV pěsnja) |
| pl | `i`-marked e (`ie` after soft-marking i) | → `ě` | niebo→něbo (~ISV nebo), wiek→věk ✓, brzeg→brěg ✓ |
| ru, be | `ё` | → `e` after C, `je` at `#`/after V (`ё < *e`) | самолёт→samolet ✓, мёд→med ✓, ёж→jež ✓ |

**Failure class (accepted):** the source marks *phonemic palatalization*, not
etymological jat, so genuine \*e after a soft consonant also becomes `ě`
(день→děnj vs ISV denj; niebo→něbo vs ISV nebo). This is the deliberate
trade-off: the marking is deterministic and honest to the *source phonology*,
`ě→e` folding is everywhere available downstream (`ascii_skeleton`), and the
dedup fold compensates (§6). South-Slavic Ekavian `e` and Bulgarian
я-alternation jat are **not** recovered (Class C, §8): the rule fires only
where the source orthography carries the palatalization signal.

Plain `e` in every language (ru `э`, uk/bg/mk/sl/sr `е`, pl/cs/sk/sh `e`
unmarked) → `e`.

---

## 4. Per-language rule tables

Ordered; digraphs before single letters; class in brackets. Languages absent
from the raw corpus but present in evidence displays (hr, sr, bs) reuse the
sh table. Base Cyrillic letters shared by all Cyrillic languages (`а б в д ж
з к л м н о п р с т у ф ц ч ш → a b v d ž z k l m n o p r s t u f c č š`,
`й/ј→j`, `х→h` `[NORM]`) are not repeated per table.

### 4.1 Russian (ru) — 49,198 raw lemmas

Extends `russian_translit.rs`; differences flagged.

| Rule | Class | Example | Notes / failure class |
|---|---|---|---|
| `г→g` | A | город→gorod | |
| `е`: `C_→ě`, else `je` | B | дело→dělo; ель→jelj | §3; **differs from translit (e/je)** |
| `ё`: `C_→e`, else `je` | B | самолёт→samolet; ёж→jež | ё < \*e; loans (сёгун) fail; **differs from translit (o/jo)** |
| `э→e` | A | этаж→etaž | |
| `и→i`, `ы→y` | A | | |
| `ю`: → `ju` everywhere | B | бюро→bjuro | **differs from translit** (`u` after C); preserves the soft signal as `j` |
| `я`: → `ja` everywhere | B | буря→burja ✓, земля→zemlja ✓ | ditto; мясо→mjaso (~ISV meso) accepted |
| `щ→šč` | A | щука→ščuka ✓ | |
| `ль`, `нь` (final or before C) → `lj`, `nj` | B | соль→solj ✓, конь→konj ✓, деньги→denjgi | matches ISV lj/nj; **differs from translit (drop ь)** |
| other `Cь` → `C` (drop) | A | кровать→krovat | ISV t́→t fold |
| `ъ` → drop (separator context handled by je/ja/ju) | A | объект→objekt ✓ | |
| verb `-ть→-ti`, adj `-ый/-ий→-y` | B-morph | читать→čitati ✓, русский→russky | §2.2; geminate kept (russky vs ISV rusky flagged §9) |

### 4.2 Ukrainian (uk) — 25,135

| Rule | Class | Example | Notes |
|---|---|---|---|
| `г→h`, `ґ→g` | A | голова→holova | \*g→h NOT reversed (§8) |
| `и→y`, `і→i`, `ї→ji` | B | риба→ryba ✓ | uk и < \*y/\*i merger; minority \*i-words surface as y |
| `е→e`; `є`: `C_→ě`, else `je` | B | небо→nebo ✓ | §3 |
| `’` (apostrophe) → `j` | A | м'ясо→mjaso | |
| `ю/я → ju/ja` | B | Юрій→Jurij | as ru |
| `ль/нь` final/pre-C → `lj/nj`; other `ь` drops | B | день→denj ✓ | uk е is hard so no false ě here |
| adj `-ий→-y`; verb `-ти` already ✓ | B-morph | добрий→dobry ✓ | |

Ikavism (`кінь`, і < \*o/\*ě in closed syllables) is Class C — `кінь→kinj`
stays, not `konj` (§8).

### 4.3 Belarusian (be) — 4,652

| Rule | Class | Example | Notes |
|---|---|---|---|
| `г→h`, `ў→v` | A/B | воўк→vovk | ў < \*v/\*l ambiguous; `v` chosen (~uk vovk); \*l cases fail (ISV vȯlk) |
| `і→i`, `ы→y`, `э→e` | A | | |
| `е`: `C_→ě`, else `je`; `ё` as ru | B | лес→lěs ✓ | §3 |
| `дз` + front (`е/і/ь/ю/я`) → `d` + front; `ц` + front → `t` + front | B | дзень→děnj (~denj), цень→těnj ✓ | dzekanne/cekanne reversal — reliable because \*c/\*dz before front is spelled `цэ/дз` + hard vowel; loans fail |
| `ль/нь` rule; `ю/я→ju/ja` as ru | B | | |
| verb `-ць→-ti` (be infinitive ending) | B-morph | чытаць→čytati | akanne not reversed (§8): галава→halava stays |

### 4.4 Polish (pl) — 65,185 (the largest corpus language)

Reference: `[VM]` pl_slo + `[NORM]`, display-tuned.

| Rule | Class | Example | Notes / failure class |
|---|---|---|---|
| `w→v` | A | winyl→vinyl ✓, woda→voda ✓ | the motivating example |
| `ch→h`, `h→h` | A | chleb→hleb (~ISV hlěb) | |
| `cz→č`, `sz→š`, `szcz→šč`, `ż→ž`, `dż→dž` | A | szczur→ščur | |
| `ó→o` | A | góra→gora ✓ | ó < \*o |
| `ł→l` | A | łapa→lapa | |
| `ą→u` | B | wąż→vuž (~ISV už), dąb→dub ✓ | ISV ų→u fold; word-initial prothetic w- not stripped (§8) |
| `ę→e` | B | imię→ime ✓, pięć→pět (~ISV pet) | right for \*ę; fails for \*ǫ-grade (ręka→reka vs ISV ruka) — accepted, flagged |
| `rz` + `e` → `rě`; `rz` else → `r` | B | rzeka→rěka ✓, przed→prěd ✓, dobrze→dobrě ✓, przy→pri ✓, brzeg→brěg ✓ | rz < \*ŕ always; morze→morě (~ISV morje) is the known miss |
| `ci/dzi/si/zi/ni` + vowel → `t/d/s/z/n` + vowel (soft-marker `i` deleted; a following `e` takes §3: `→ě`) | B | niebo→něbo (~nebo), ciało→talo (~tělo), nici→niti ✓ | de-palatalization to the etymological stop `[VM]`; before C or `#` the `i` is a real vowel and stays (`ti/di/si/zi/ni`) |
| `ć/dź/ś/ź/ń` (final or pre-C) → `t/d/s/z/n`; final `ń→nj` | B | radość→radost ✓, koń→konj ✓ | ISV t́/d́/ś/ź/ń folds `[ORTH]`; verb `-ć` already consumed by §2.2 (być→byti ✓) |
| `ie` after other consonants (labials etc.) → `ě` | B | wiek→věk ✓, niebo→něbo (~nebo) | §3 |
| `ia/io/iu` (soft marker) → `ja/jo/ju` | B | biały→bjaly, wiara→vjara (~ISV věra) | przegłos (`ia < *ě`) NOT reversed — needs etymology (§8) |
| `y→y` | A | winyl→vinyl ✓ | **not** folded to i (voting folds it; display must not) |
| adj `-y` already ✓; verb `-ć→-ti` §2.2 | | | |

### 4.5 Czech (cs) — 33,555

| Rule | Class | Example | Notes |
|---|---|---|---|
| `á é í ó ú ý → a e i o u y` | A | být→byt→(verb §2.2)→byti ✓ | length is noise `[VM]` |
| `ů→o` | B | kůň→konj ✓, dům→dom ✓ | ů < \*o; loans in -ů- rare |
| `ě→ě` | A | město→město ✓ | kept verbatim §3 |
| `ch→h` | A | chyba→hyba (~ISV hyba ✓) | cs h NOT →g (§8) |
| `ou→u` | B | mouka→muka ✓, soud→sud ✓ | ou < \*u/\*ǫ; loans (kouč) fail |
| `ř`+`e`→`rě`, else `ř→r` | B | řeka→rěka ✓, tři→tri ✓, dobře→dobrě ✓ | moře→morě (~morje) known miss, as pl |
| `ď/ť/ň` final/pre-C → `d/t/n`; final `ň→nj` | B | loď→lod, kůň→konj ✓ | spelling `ďa ťa ňa → dja tja nja`? no — → `d't'n` + `ja` is not attested in lemmas; `ďa→dja` (B, rare) |
| `w→v`, `x→ks`, `q→k` | A (post-pass) | | |
| verb `-t→-ti`, adj `-ý→-y` | B-morph | dělat→dělati ✓, nový→novy ✓ | |

### 4.6 Slovak (sk) — 7,312

As Czech, plus:

| Rule | Class | Example | Notes |
|---|---|---|---|
| `ä→e` | B | mäso→meso ✓ | ä < \*ę |
| `ô→o` | B | kôň→konj ✓ | |
| `ie→ě`, `ia→ja`, `iu→ju` | B | viera→věra ✓ | §3; ia < \*ę́ not recovered (accepted) |
| `ľ→lj` (final/pre-V), `ľ` pre-C → `l`; `ĺ→l`, `ŕ→r` | B | ľud→ljud ✓ | |
| `dz→dz`, `dž→dž` | A | medzi→medzi | dz < \*dj NOT rewritten to dž (mixed loan class; §9 open question) |
| verb `-ť→-ti` | B-morph | robiť→robiti ✓ | |

### 4.7 Slovenian (sl) — 3,959

Nearly identity: `č š ž lj nj j v h` are already ISV-compatible; strip accent
marks (pre-pass); `w→v x→ks q→k` post-pass. Verb `-ti` ✓ already. No jat, no
`y` (merged to i — not recoverable, §8), no nasal recovery (golob stays).

### 4.8 Serbo-Croatian (sh, + hr/bs Latin, sr via Cyrillic) — 52,138

Serbian Cyrillic first maps ђ→đ ћ→ć џ→dž љ→lj њ→nj ј→j (Class A), then:

| Rule | Class | Example | Notes |
|---|---|---|---|
| `đ→dž`, `ć→č` | A | vođa→vodža, noć→noč ✓ | the §1.3 standard fold; keeps vođa/voda distinct (dedup invariant) |
| `ije→ě`, `Cje→Cě` (`#je` stays `je`) | B | rijeka→rěka ✓, mjesto→město ✓, jezik→jezik ✓ | Ijekavian only; loans (objekt) fail — flagged; Ekavian e not recovered (§8) |
| `dž lj nj š ž č r`-syllabic | A | prst→prst ✓ | pass through |
| adj `-i→-y` | B-morph | novi→novy ✓ | definite citation form |

### 4.9 Bulgarian (bg) — 15,493

| Rule | Class | Example | Notes |
|---|---|---|---|
| `щ→št` | A | нощ→nošt (~ISV noč) | bg-specific `[NORM]`; \*tj→ISV č NOT applied (Class C — spelling can't separate \*tj from \*st-clusters) |
| `ъ→o` | B | дъжд→dožd ✓ | ъ = strong-yer reflex → ISV ȯ→o `[NORM]`; \*ǫ-words fail (ръка→roka vs ruka) — accepted, flagged |
| `е→e`, `я→ja`, `ю→ju` | A/B | поляна→poljana | jat-alternation я (бял) NOT →ě (§8) |
| `ь→j` (only in `ьо`) | A | синьо→sinjo | |
| verbs | — | чета stays četa | no infinitive in bg (§2.2) |

### 4.10 Macedonian (mk) — 39,652

| Rule | Class | Example | Notes |
|---|---|---|---|
| `ќ→č`, `ѓ→dž` | B | ноќ→noč ✓, меѓу→medžu ✓ | ќ/ѓ < \*tj/\*dj — the ISV `ć/đ` letters, standard-folded; loans (ѓеврек) fail |
| `ѕ→dz`, `џ→dž`, `љ→lj`, `њ→nj`, `ј→j` | A | ѕвезда→dzvezda (~ISV zvězda) | |
| `е→e`, `и→i` | A | | no y (merged; §8) |
| verbs | — | чита stays čita | no infinitive in mk (§2.2) |

### 4.11 Sorbian (hsb 807, dsb 1,354)

Polish-like `[VM]`-family orthography: `w→v ó→o ł→l č š ž ě` (**ě kept**, §3),
`ch→h`; hsb `ř`+e→`rě` else `r` (přez→prěz ✓); `dź→d`+§3 vowel (dźěło→dělo ✓);
dsb `ś ź → s z`; `ć` final → `t` (verb `-ć→-ti` per §2.2); `y→y`. Class B
throughout; small corpora, best-effort.

### 4.12 Silesian (szl, 2,012) and Kashubian (csb, 2,328)

Polish table plus vowel extras — szl: `ō ô ŏ → o o o`, `ů→o` (gůra→gora ✓);
csb: `ë→e`, `ò→o`, `ô→o`, `ù→u`, `é→e`, `ã→e` (via \*ę), `ą→u`. Class B,
best-effort, residue-counted.

### 4.13 Old Church Slavonic (cu, 3,198) and Old East Slavic (orv, 560)

`[NORM]` Cyrillic table plus: `ѣ→ě`, `ѧ→e` (ę-fold), `ѫ→u` (ǫ-fold), `ѩ→je`,
`ѭ→ju`, `оу/ѹ→u`, `ꙑ→y`, `ѳ→f`, `ѵ→i`, `щ→št`. Yers by Havlík's law
(deterministic from spelling, `RULE_SPEC.md` §2 assign-yer-strength): weak →
drop, strong → `o`/`e` (сънъ→son ✓, дьнь→den ✓). Best-effort; these are
etymological-hint languages with tiny page counts.

---

## 5. Deliberate differences from the voting normalizer (`normalize.rs`)

| Dimension | Voting (`to_phonemic_latin`) | Display (`flavorize_word`) | Why |
|---|---|---|---|
| Purpose | cognate alignment key | reader-facing headword | benchmark-gated vs display-only |
| Case | lowercases | preserves | display |
| Output alphabet | phonemic Latin incl. `ę ǫ đ ř` | ISV standard only (§1) | ę/ǫ/đ/ř are not ISV standard letters |
| pl `rz` | `→ř` | `→r/rě` | ř is not an ISV letter |
| pl nasals | kept `ę/ǫ` | folded `e/u` | etymological signal vs standard spelling |
| ru `е` | `→e/je` | `→ě/je` | §3 (the palatalization signal is the point) |
| ru `ё` | `→o/jo` | `→e/je` | display flavorizes toward \*e |
| Endings | untouched | POS-adapted (§2.2) | citation-form alignment |
| Failure mode | silent (vote noise) | counted + reported (§2 stage 5) | loud-failure policy |

Any change to `normalize.rs` remains benchmark-gated per CONTRIBUTING.md and
is **not** part of this feature.

---

## 6. Site integration contract

- **Display = dedup, in lockstep.** `raw_lemma_fate` (site.rs) must derive its
  fold from the *same* `flavorize_word` output used as the display headword —
  today both call `source_display`; both switch together. Flavorization
  *improves* dedup: `winyl→vinyl`, `дело→dělo` now collide with the official
  pages they orthographically are, which is the intended fold (the `konflikt`
  precedent in `raw_lemma_fate`'s doc comment).
- **ě-tolerant dedup.** Because §3 can over-mark `ě` (děnj vs official denj),
  the raw-vs-official check must test both `to_standard(display)` **and** its
  `ě→e` fold against `isv_to_id`; raw-vs-raw dedup keys on the `ě→e` fold so
  the same word attested in cs (`ě`) and sr (`e`) collapses to one page.
- **The attested form stays primary evidence.** The infobox "Atestovana
  forma", source URL, and search alias slot (row element 12 — verbatim
  original + Latin fold) keep the untouched national spelling; a query for
  `winyl` or `пластинка` must still find the page (already guaranteed by the
  #31 alias path).
- **No cache change, no schema bump.** Flavorization runs at render time from
  `RawSlavicLemma.word`; extractors and committed caches are untouched.
- **Untouched surfaces:** forms/verification API (byte-identical), benchmark
  (`evaluate` never reads the raw path), `normalize.rs`, running-text
  transliteration.

---

## 7. Worked examples (golden-test seed)

| lang | attested | today | flavorized | ISV official (if any) |
|---|---|---|---|---|
| pl | winyl | winyl | vinyl | — |
| pl | rzeka | rzeka | rěka | rěka ✓ |
| pl | radość | radość | radost | radost ✓ |
| pl | być | być | byti | byti ✓ |
| cs | dělat | dělat | dělati | dělati ✓ |
| cs | kůň | kůň | konj | konj ✓ |
| cs | mouka | mouka | muka | mųka→muka ✓ |
| sk | mäso | mäso | meso | męso→meso ✓ |
| sl | delati | delati | delati | dělati (~e) |
| sh | rijeka | rijeka | rěka | rěka ✓ |
| sh | noć | noć | noč | noč ✓ |
| sr (cyr) | међа | међа | medža | medža ✓ |
| mk | меѓу | меѓу | medžu | medžu ✓ |
| mk | ноќ | ноќ | noč | noč ✓ |
| bg | дъжд | дъжд | dožd | dȯžd→dožd ✓ |
| ru | пластинка | plastinka | plastinka | — |
| ru | дело | delo | dělo | dělo ✓ |
| ru | самолёт | samolot | samolet | samolet ✓ |
| ru | читать | čitat | čitati | čitati ✓ |
| ru | конь | kon | konj | konj ✓ |
| uk | голова | голова | holova | golova (h not reversed — §8) |
| be | цень | цень | těnj | těnj ✓ |
| hsb | dźěło | dźěło | dělo | dělo ✓ |
| cu | дьнь | дьнь | den | denj (~nj) |

(`today` column: ru via `russian_translit`, everything else verbatim.)

---

## 8. Class C ledger — explicitly NOT attempted (needs etymology)

| Phenomenon | Languages | Example that stays "wrong" | Layer-2 hook |
|---|---|---|---|
| \*g → h restoration | cs, sk, uk, be, hsb, rue | holova ≠ golova | `spirantize_h_to_g` needs cognates |
| Akanne | be | halava ≠ golova | vowel identity lost in spelling |
| Ikavism | uk | kinj ≠ konj | і < \*o/\*e/\*ě ambiguous |
| Ekavian/ekavica jat | sr, mk, bg unstressed | reka ≠ rěka | no spelling signal |
| bg я jat-alternation | bg | bjal ≠ běl | ja vs jat needs cognates |
| Polish przegłos | pl | bjaly ≠ běly, talo ≠ tělo | ia < \*ě vs \*ja |
| Nasal prothesis / \*ǫ-grade | pl, bg | vuž ≠ už, reka(ręka) ≠ ruka | `nasal_from_polish` exists for voting |
| Pleophony | ru, uk, be | gorod stays gorod (ISV gråd/grad context-dependent) | `undo_pleophony` is a voting rule |
| y/i merger | sl, sh, mk, bg | novi→novy is morph-only; stem i stays | no signal |

These are exactly the phenomena the consensus engine resolves with cognate
sets; a raw word has none. If a raw word later joins a generated cognate set,
it gets a real generated page and leaves the raw path entirely — that, not
Layer-2 cleverness, is the preferred fix.

## 9. Open questions

1. sk/pl `dz` < \*dj (medzi): rewrite to `dž` (medžu-ward) or keep `dz`?
   Kept for now; revisit with a frequency count.
2. Geminate collapse (russky→rusky, vanna): skipped — needs a loan-aware
   exception list.
3. Reflexive citation forms (`-ся` verbs surviving the single-token gate):
   adapt to `X sę` or keep? Currently keep.
4. ru `-чь`/cs `-ct`/pl `-c` infinitives (moč/moct/móc vs ISV mogti):
   excluded from §2.2; a tiny closed list could cover them.
5. Should evidence-form displays inside *generated* entries (site.rs
   `source_display` call sites beyond the raw path) flavorize too, or only
   raw headwords? Proposed: yes for word-chips, no for running text (§0).
