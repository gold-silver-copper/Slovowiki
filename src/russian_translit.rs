//! Deterministic Russian Cyrillic → Interslavic-style Latin transliteration for
//! site display.
//!
//! This is intentionally a script transliterator, not a translator. It runs as
//! part of static site generation whenever Russian Wiktionary text is rendered,
//! so generated pages never show raw Russian Cyrillic while source URLs still
//! point at the original Russian entries.

/// Transliterate Russian text to the same broad Latin/Interslavic conventions
/// used elsewhere on the site: ж→ž, ч→č, ш→š, щ→šč, х→h, ц→c, ю/я→ju/ja or
/// u/a after consonants, and hard/soft signs omitted.
pub fn to_interslavic(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len());
    for (i, &ch) in chars.iter().enumerate() {
        // Drop stress/combining marks common in Wiktionary Russian headwords.
        if ('\u{0300}'..='\u{036F}').contains(&ch) {
            continue;
        }
        let prev = previous_base(&chars, i);
        let lower = ch.to_lowercase().next().unwrap_or(ch);
        let repl = match lower {
            'а' => "a",
            'б' => "b",
            'в' => "v",
            'г' => "g",
            'д' => "d",
            'е' => {
                if prev == Some('ь') || prev == Some('ъ') || !is_after_consonant(prev) {
                    "je"
                } else {
                    "e"
                }
            }
            'ё' => {
                if prev == Some('ь') || prev == Some('ъ') || !is_after_consonant(prev) {
                    "jo"
                } else {
                    "o"
                }
            }
            'ж' => "ž",
            'з' => "z",
            'и' => "i",
            'й' => "j",
            'к' => "k",
            'л' => "l",
            'м' => "m",
            'н' => "n",
            'о' => "o",
            'п' => "p",
            'р' => "r",
            'с' => "s",
            'т' => "t",
            'у' => "u",
            'ф' => "f",
            'х' => "h",
            'ц' => "c",
            'ч' => "č",
            'ш' => "š",
            'щ' => "šč",
            'ъ' | 'ь' => "",
            'ы' => "y",
            'э' => "e",
            'ю' => {
                if prev == Some('ь') || prev == Some('ъ') || !is_after_consonant(prev) {
                    "ju"
                } else {
                    "u"
                }
            }
            'я' => {
                if prev == Some('ь') || prev == Some('ъ') || !is_after_consonant(prev) {
                    "ja"
                } else {
                    "a"
                }
            }
            _ => {
                out.push(ch);
                continue;
            }
        };
        if ch.is_uppercase() {
            push_capitalized(&mut out, repl);
        } else {
            out.push_str(repl);
        }
    }
    out
}

fn previous_base(chars: &[char], i: usize) -> Option<char> {
    chars[..i]
        .iter()
        .rev()
        .copied()
        .find(|c| !(('\u{0300}'..='\u{036F}').contains(c)))
        .map(|c| c.to_lowercase().next().unwrap_or(c))
}

fn is_after_consonant(prev: Option<char>) -> bool {
    let Some(prev) = prev else { return false };
    if !prev.is_alphabetic() || prev == 'ь' || prev == 'ъ' {
        return false;
    }
    !matches!(
        prev,
        'а' | 'е' | 'ё' | 'и' | 'о' | 'у' | 'ы' | 'э' | 'ю' | 'я'
    )
}

fn push_capitalized(out: &mut String, repl: &str) {
    let mut chars = repl.chars();
    if let Some(first) = chars.next() {
        for up in first.to_uppercase() {
            out.push(up);
        }
    }
    out.extend(chars);
}

#[cfg(test)]
mod tests {
    use super::to_interslavic as tr;

    #[test]
    fn transliterates_russian_examples() {
        assert_eq!(tr("вода́"), "voda");
        assert_eq!(tr("русский язык"), "russkij jazyk");
        assert_eq!(tr("семья, объект"), "semja, objekt");
        assert_eq!(tr("ёлка и щука"), "jolka i ščuka");
    }

    #[test]
    fn preserves_basic_capitalization() {
        assert_eq!(tr("Россия"), "Rossija");
        assert_eq!(tr("Юрий"), "Jurij");
    }
}
