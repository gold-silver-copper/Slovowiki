//! Interslavic orthography helpers.
//!
//! Interslavic has two written standards. The official dictionary stores lemmas
//! in the *flavored / scientific* alphabet, which preserves etymological
//! distinctions (jat `ě`, nasals `ę`/`ų`, the liquid-diphthong vowel `å`, the
//! yer reflexes `ȯ`/`ė`, the soft consonants `ĺ ń ŕ ť ď ś ź`, and `ć`/`đ`).
//! The *standard* alphabet folds those away. We keep helpers for both, plus the
//! aggressive ASCII skeleton used to align cognates and to compute the
//! "normalized" match metric.

/// Fold the flavored/scientific alphabet down to the *standard* Interslavic
/// alphabet. This is the officially defined "loss of flavor" mapping and is the
/// basis of the normalized match metric: two spellings that only differ in
/// etymological flavor collapse to the same standard string.
pub fn to_standard(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    for ch in word.chars() {
        match ch {
            'ě' => out.push('e'),
            'ę' => out.push('e'),
            'ų' => out.push('u'),
            'å' => out.push('a'),
            'ȯ' => out.push('o'),
            'ė' => out.push('e'),
            // §1.3: the soft-consonant etymological letters simply drop their
            // diacritic in the standard alphabet (ĺ→l, ń→n, …), while ć/đ become
            // č/dž.
            'ĺ' => out.push('l'),
            'ľ' => out.push('l'),
            'ń' => out.push('n'),
            'ŕ' => out.push('r'),
            'ť' => out.push('t'),
            'ď' => out.push('d'),
            'ś' => out.push('s'),
            'ź' => out.push('z'),
            'ć' => out.push('č'),
            'đ' => out.push_str("dž"),
            'Ě' => out.push('E'),
            'Ę' => out.push('E'),
            'Ų' => out.push('U'),
            'Å' => out.push('A'),
            'Ȯ' => out.push('O'),
            other => out.push(other),
        }
    }
    out
}

/// Aggressive ASCII skeleton: strip *all* diacritics and fold the phonemically
/// close consonant classes together. Used to align cognates across languages
/// and as a looser matching key. Preserves the y/i and hard/soft distinctions
/// only where they survive as separate ASCII letters.
pub fn ascii_skeleton(word: &str) -> String {
    let std = to_standard(&word.to_lowercase());
    let mut out = String::with_capacity(std.len());
    for ch in std.chars() {
        match ch {
            'č' | 'ć' | 'ç' => out.push('c'),
            'š' | 'ś' | 'ş' => out.push('s'),
            'ž' | 'ź' | 'ż' => out.push('z'),
            'đ' => out.push('d'),
            'ń' | 'ň' => out.push('n'),
            'ľ' | 'ĺ' | 'ł' => out.push('l'),
            'ř' | 'ŕ' => out.push('r'),
            'ť' => out.push('t'),
            'ď' => out.push('d'),
            'á' | 'à' | 'â' | 'ā' | 'ǎ' | 'å' => out.push('a'),
            'é' | 'è' | 'ê' | 'ē' | 'ě' | 'ę' => out.push('e'),
            'í' | 'ì' | 'î' | 'ī' => out.push('i'),
            'ó' | 'ò' | 'ô' | 'ō' | 'ȯ' | 'ǫ' => out.push('o'),
            'ú' | 'ù' | 'û' | 'ū' | 'ů' | 'ų' => out.push('u'),
            'ý' | 'ỳ' | 'ŷ' | 'ȳ' => out.push('y'),
            other => out.push(other),
        }
    }
    out
}

/// Consonant-only alignment key for voting *within one meaning group*, where all
/// forms are cognate by construction. Drops vowels and semivowels, folds the
/// regular consonant correspondences that split cognates across branches
/// (notably *g→h in Czech/Slovak/Ukrainian/Belarusian, and the sibilant
/// classes). The result is a compact fingerprint that collapses pleophony,
/// vowel-quality shifts, and nasal/jat differences so that true cognates across
/// East/West/South land on the same key.
pub fn consonant_key(word: &str) -> String {
    let skel = ascii_skeleton(word);
    let mut out = String::with_capacity(skel.len());
    let mut prev = '\0';
    for ch in skel.chars() {
        let mapped = match ch {
            // vowels and semivowels: dropped
            'a' | 'e' | 'i' | 'o' | 'u' | 'y' | 'j' | '\'' | 'ь' | 'ъ' => '\0',
            // *g and *x both surface as h in several languages; merge to g so
            // cognates align (over-merge is safe inside one meaning group).
            'h' => 'g',
            'w' => 'v',
            'ł' => 'l',
            other => other,
        };
        if mapped != '\0' && mapped != prev {
            out.push(mapped);
            prev = mapped;
        } else if mapped == '\0' {
            prev = '\0';
        }
    }
    out
}

/// Case-insensitive exact equality on the flavored spelling.
pub fn exact_match(a: &str, b: &str) -> bool {
    a.trim().to_lowercase() == b.trim().to_lowercase()
}

/// Match after folding both sides to the standard alphabet.
pub fn normalized_match(a: &str, b: &str) -> bool {
    to_standard(&a.trim().to_lowercase()) == to_standard(&b.trim().to_lowercase())
}

/// Match on the aggressive ASCII skeleton (loosest).
pub fn skeleton_match(a: &str, b: &str) -> bool {
    ascii_skeleton(a) == ascii_skeleton(b)
}

/// Levenshtein edit distance over Unicode scalar values.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for (i, &ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            cur[j + 1] = (prev[j + 1] + 1).min(cur[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

/// Normalized edit distance in [0,1]: edit distance over the standard spelling
/// divided by the longer length.
pub fn normalized_edit_distance(a: &str, b: &str) -> f32 {
    let sa = to_standard(&a.to_lowercase());
    let sb = to_standard(&b.to_lowercase());
    let d = levenshtein(&sa, &sb);
    let len = sa.chars().count().max(sb.chars().count()).max(1);
    d as f32 / len as f32
}

pub fn is_vowel(ch: char) -> bool {
    matches!(
        ch,
        'a' | 'e' | 'i' | 'o' | 'u' | 'y'
            | 'ě' | 'ę' | 'ų' | 'å' | 'ȯ' | 'ė'
            | 'á' | 'é' | 'í' | 'ó' | 'ú' | 'ý'
    )
}
