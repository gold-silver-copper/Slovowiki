//! Morphophonemics shared across the crate — the single home for the
//! palatalization and iotation correspondence tables and the softness
//! predicate (issue #15).
//!
//! Before this module existed the crate carried three divergent copies:
//! `derive.rs` (full tables), `forms.rs` (7-pair subsets whose softness set
//! was also a subset of the canonical one — soft stems in `-ń`/`-ľ` got the
//! hard comparative ending `-ějši`), and the transposed inverse tables in
//! `derive.rs`. Everything now reads the two tables below; the inverse
//! lookups transpose them programmatically.

/// First palatalization at a suffix seam (RULE_SPEC §2: live before
/// `-ny, -ka/-ko/-ok, -sky, -stvo, -ec, -ica, -ina, -išče, -nik`, and the
/// comparative `-ejši`).
pub const PALATALIZATION: &[(char, char)] = &[('k', 'č'), ('g', 'ž'), ('h', 'š'), ('c', 'č')];

/// Iotation of a stem-final consonant before a `-je-` suffix (RULE_SPEC §2
/// Phase D): s→š, z→ž, t→ć, d→đ, st→šć, zd→žđ, k→č, g→ž, h→š, labials take
/// bare j (lovjeńje), sonorants soften (děljeńje). Longest match first.
pub const IOTATION: &[(&str, &str)] = &[
    ("st", "šć"),
    ("zd", "žđ"),
    ("s", "š"),
    ("z", "ž"),
    ("t", "ć"),
    ("d", "đ"),
    ("k", "č"),
    ("g", "ž"),
    ("h", "š"),
    ("l", "lj"),
    ("n", "nj"),
    ("r", "rj"),
    ("p", "pj"),
    ("b", "bj"),
    ("v", "vj"),
    ("m", "mj"),
];

/// A stem counts as soft (for O⇒E ending alternation and the comparative
/// `-ejši`/`-ějši` choice). THE softness definition for the whole crate.
pub fn is_soft(stem: &str) -> bool {
    let last = stem.chars().last().unwrap_or(' ');
    matches!(
        last,
        'š' | 'ž' | 'č' | 'c' | 'j' | 'ć' | 'đ' | 'ń' | 'ľ' | 'ŕ'
    ) || stem.ends_with("lj")
        || stem.ends_with("nj")
        || stem.ends_with("dž")
}

/// Apply first palatalization to a stem-final consonant.
pub fn palatalize_final(stem: &str) -> String {
    let mut s = stem.to_string();
    if let Some(last) = s.chars().last() {
        if let Some((_, soft)) = PALATALIZATION.iter().find(|(hard, _)| *hard == last) {
            s.pop();
            s.push(*soft);
        }
    }
    s
}

/// Apply iotation to a stem-final consonant (longest match first).
pub fn iotate_final(stem: &str) -> String {
    for (suf, rep) in IOTATION {
        if let Some(head) = stem.strip_suffix(suf) {
            return format!("{head}{rep}");
        }
    }
    stem.to_string()
}

/// All possible un-palatalized sources of a stem (includes the stem itself,
/// since hushing-final stems can be original).
pub fn inverse_palatalization(stem: &str) -> Vec<String> {
    let mut v = vec![stem.to_string()];
    for (hard, soft) in PALATALIZATION {
        if let Some(head) = stem.strip_suffix(*soft) {
            v.push(format!("{head}{hard}"));
        }
    }
    v
}

/// All possible un-iotated sources of a form (includes the unchanged form so
/// hushing-final stems — učiti → uč- — resolve too). Longest match first,
/// covering every hard source of an ambiguous soft outcome (š ← s or h,
/// ž ← z or g, č ← k …).
pub fn inverse_iotation(t: &str) -> Vec<String> {
    let mut v = vec![t.to_string()];
    for (hard, soft) in IOTATION {
        if let Some(head) = t.strip_suffix(soft) {
            v.push(format!("{head}{hard}"));
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_round_trip() {
        // Every iotation outcome inverts back to its source.
        for (hard, soft) in IOTATION {
            let stem = format!("xx{hard}");
            let iotated = iotate_final(&stem);
            // Longest-match: single-char sources inside a longer match (st)
            // iotate as the longer pair; skip those in the round-trip.
            if iotated == format!("xx{soft}") {
                assert!(
                    inverse_iotation(&iotated).contains(&stem),
                    "{stem} → {iotated} does not invert"
                );
            }
        }
        for (hard, _) in PALATALIZATION {
            let stem = format!("xx{hard}");
            assert!(inverse_palatalization(&palatalize_final(&stem)).contains(&stem));
        }
    }

    #[test]
    fn softness_covers_all_palatal_outcomes() {
        // Any stem produced by palatalization or iotation must count as soft
        // (this is the invariant forms.rs's old local copy violated).
        for (hard, _) in PALATALIZATION {
            assert!(is_soft(&palatalize_final(&format!("xx{hard}"))));
        }
        for (hard, _) in IOTATION {
            assert!(is_soft(&iotate_final(&format!("xx{hard}"))));
        }
        // The full soft set, incl. the members forms.rs used to miss.
        for s in [
            "końń", "xxń", "xxľ", "xxŕ", "xxć", "xxđ", "xxlj", "xxnj", "xxdž",
        ] {
            assert!(is_soft(s), "{s} must be soft");
        }
    }
}
