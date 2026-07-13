//! Shared semantic matching between generated corpus candidates and official senses.
//!
//! Spelling only identifies candidate rows. A match additionally requires the
//! same part of speech and positive gloss evidence, preventing homographs from
//! becoming labels or published official matches by spelling alone.

use crate::model::{Candidate, Pos};
use crate::official::OfficialEntry;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfficialMatch {
    /// One-based rank in the generated publication candidate list.
    pub candidate_rank: usize,
    /// Index in the loaded official-entry vector.
    pub sense_index: usize,
    /// Stable official dictionary sense identifier.
    pub sense_id: String,
}

pub struct OfficialIndex {
    exact: HashMap<String, Vec<usize>>,
    folded: HashMap<String, Vec<usize>>,
}

impl OfficialIndex {
    pub fn new(entries: &[OfficialEntry]) -> Self {
        let mut exact: HashMap<String, Vec<usize>> = HashMap::new();
        let mut folded: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, entry) in entries.iter().enumerate() {
            let spelling = entry.isv.trim();
            if spelling.is_empty() || spelling.contains(' ') || spelling.contains('#') {
                continue;
            }
            let lower = spelling.to_lowercase();
            exact.entry(lower.clone()).or_default().push(i);
            folded
                .entry(crate::orthography::to_standard(&lower))
                .or_default()
                .push(i);
        }
        Self { exact, folded }
    }

    /// Resolve the first publication candidate with lexical and semantic
    /// evidence. Exact scientific spelling is preferred. A folded lookup is
    /// rejected when it aliases distinct official spellings.
    pub fn match_candidates(
        &self,
        candidates: &[Candidate],
        entries: &[OfficialEntry],
        pos: Pos,
        gloss: &str,
    ) -> Option<OfficialMatch> {
        candidates.iter().take(5).enumerate().find_map(|(rank, c)| {
            self.match_form(&c.form, entries, pos, gloss)
                .map(|sense_index| OfficialMatch {
                    candidate_rank: rank + 1,
                    sense_id: entries[sense_index].id.clone(),
                    sense_index,
                })
        })
    }

    pub fn match_form(
        &self,
        form: &str,
        entries: &[OfficialEntry],
        pos: Pos,
        gloss: &str,
    ) -> Option<usize> {
        let lower = form.trim().to_lowercase();
        let rows = if let Some(rows) = self.exact.get(&lower) {
            rows.as_slice()
        } else {
            let rows = self
                .folded
                .get(&crate::orthography::to_standard(&lower))?
                .as_slice();
            let mut spellings = rows.iter().map(|&i| entries[i].isv.trim().to_lowercase());
            let first = spellings.next()?;
            if spellings.any(|spelling| spelling != first) {
                return None;
            }
            rows
        };
        select_official_entry(rows, entries, pos, gloss)
    }

    pub fn contains_fold(&self, form: &str) -> bool {
        self.folded.contains_key(&crate::orthography::to_standard(
            &form.trim().to_lowercase(),
        ))
    }
}

/// Choose the strongest compatible sense from an already spelling-filtered row
/// set. Exact/folded spelling without this semantic evidence is never positive.
pub fn select_official_entry(
    rows: &[usize],
    entries: &[OfficialEntry],
    pos: Pos,
    gloss: &str,
) -> Option<usize> {
    let wanted = crate::dump::gloss_tokens(gloss);
    let compact = wanted.join("");
    rows.iter()
        .copied()
        .filter(|&i| entries[i].pos == pos)
        .map(|i| {
            let actual = crate::dump::gloss_tokens(&entries[i].english);
            let overlap = wanted.iter().filter(|token| actual.contains(token)).count();
            let compound = !compact.is_empty() && compact == actual.join("");
            (i, overlap, compound)
        })
        .filter(|(_, overlap, compound)| *overlap > 0 || *compound)
        .max_by_key(|(i, overlap, compound)| (*overlap, *compound, std::cmp::Reverse(*i)))
        .map(|(i, _, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{NounTraits, Pos};
    use std::collections::HashMap;

    fn entry(id: &str, isv: &str, pos: Pos, english: &str) -> OfficialEntry {
        OfficialEntry {
            id: id.into(),
            isv: isv.into(),
            addition: String::new(),
            pos_raw: pos.code().into(),
            pos,
            noun_traits: NounTraits::default(),
            english: english.into(),
            same_in: String::new(),
            genesis: String::new(),
            cells: HashMap::new(),
            frequency: None,
            de: String::new(),
            nl: String::new(),
            eo: String::new(),
            intelligibility: String::new(),
            using_example: String::new(),
        }
    }

    #[test]
    fn spelling_requires_pos_and_semantics() {
        let entries = vec![
            entry("1", "bajka", Pos::Noun, "fairytale"),
            entry("2", "bajka", Pos::Verb, "speak"),
        ];
        let index = OfficialIndex::new(&entries);
        assert_eq!(
            index.match_form("bajka", &entries, Pos::Noun, "fairy tale"),
            Some(0)
        );
        assert_eq!(
            index.match_form("bajka", &entries, Pos::Adjective, "fairy tale"),
            None
        );
        assert_eq!(
            index.match_form("bajka", &entries, Pos::Noun, "machine"),
            None
        );
    }

    #[test]
    fn ambiguous_fold_is_rejected() {
        let entries = vec![
            entry("1", "dŕžati", Pos::Verb, "hold"),
            entry("2", "držati", Pos::Verb, "hold"),
        ];
        let index = OfficialIndex::new(&entries);
        assert_eq!(
            index.match_form("drzati", &entries, Pos::Verb, "hold"),
            None
        );
        assert_eq!(
            index.match_form("dŕžati", &entries, Pos::Verb, "unrelated"),
            None
        );
    }
}
