//! Cognate-set dictionary built from the Wiktionary Slavic-lemma corpus.
//!
//! Every inherited Slavic lemma Wiktionary links to a Proto-Slavic ancestor
//! ([`crate::dump::extract_lemmas`]); lemmas sharing an ancestor form a **cognate
//! set**. Each set becomes one Interslavic word: the Proto-Slavic rule engine
//! supplies the form from the *known* reconstruction (no linking needed), the
//! modern reflexes resolve the yers and give the surface consensus, and the
//! **confidence scales with how many languages and branches attest the set** —
//! a word seen in one language is a low-confidence guess, one seen across all
//! three branches is high-confidence.

use crate::consensus::{self, ConsensusConfig, MeaningInput, SourceForm};
use crate::dump::{LemmaCorpus, LemmaEntry};
use crate::lang::Branch;
use crate::model::{Candidate, Confidence, Pos};
use crate::normalize::{self, NormForm};
use crate::orthography as ortho;
use std::collections::BTreeMap;

/// A group of etymologically-connected modern lemmas (shared Proto-Slavic root).
#[derive(Debug, Clone)]
pub struct CognateSet {
    pub proto: String,
    pub pos: Pos,
    pub gloss: String,
    pub members: Vec<LemmaEntry>,
}

/// One generated Interslavic word plus its supporting cognate set.
pub struct GeneratedWord {
    pub set: CognateSet,
    pub candidates: Vec<Candidate>,
    pub confidence: Confidence,
    pub score: f32,
    pub n_langs: usize,
    pub n_branches: usize,
    pub reconstruction: Option<crate::model::Reconstruction>,
}

impl GeneratedWord {
    pub fn form(&self) -> &str {
        self.candidates
            .first()
            .map(|c| c.form.as_str())
            .unwrap_or("")
    }
}

/// Branch of a Slavic language, including the smaller lects the corpus carries.
pub fn branch_of(lang: &str) -> Option<Branch> {
    Some(match lang {
        "ru" | "uk" | "be" | "rue" => Branch::East,
        "pl" | "cs" | "sk" | "dsb" | "hsb" | "csb" | "szl" => Branch::West,
        "sl" | "hr" | "sr" | "bs" | "bg" | "mk" | "cu" => Branch::South,
        _ => return None,
    })
}

fn pos_class(pos: &str) -> &'static str {
    match pos {
        "noun" | "proper_noun" => "n",
        "verb" => "v",
        "adj" => "a",
        "adv" => "adv",
        "pron" => "pron",
        "num" => "num",
        "prep" => "prep",
        "conj" => "conj",
        _ => "x",
    }
}

/// Group the corpus into cognate sets keyed by (Proto-Slavic ancestor, POS class).
pub fn build_sets(corpus: &LemmaCorpus) -> Vec<CognateSet> {
    let mut groups: BTreeMap<(String, &'static str), Vec<LemmaEntry>> = BTreeMap::new();
    for e in &corpus.entries {
        if branch_of(&e.lang).is_none() {
            continue;
        }
        groups
            .entry((e.proto.clone(), pos_class(&e.pos)))
            .or_default()
            .push(e.clone());
    }

    let mut sets = Vec::new();
    for ((proto, _), mut members) in groups {
        // Dedupe identical (language, word) senses.
        members.sort_by(|a, b| (&a.lang, &a.word).cmp(&(&b.lang, &b.word)));
        members.dedup_by(|a, b| a.lang == b.lang && a.word == b.word);
        if members.is_empty() {
            continue;
        }
        let pos = most_common_pos(&members);
        let gloss = representative_gloss(&members);
        sets.push(CognateSet {
            proto,
            pos,
            gloss,
            members,
        });
    }
    sets
}

fn most_common_pos(members: &[LemmaEntry]) -> Pos {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for m in members {
        *counts.entry(m.pos.as_str()).or_default() += 1;
    }
    counts
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map(|(p, _)| Pos::parse(p))
        .unwrap_or(Pos::Other)
}

/// The gloss shared by the most members (the cognate-set's meaning), preferring a
/// major reference language on ties.
fn representative_gloss(members: &[LemmaEntry]) -> String {
    const PREF: &[&str] = &["ru", "pl", "cs", "uk", "sl", "bg"];
    let mut counts: BTreeMap<&str, (usize, i32)> = BTreeMap::new();
    for m in members {
        let g = m.gloss.trim();
        if g.is_empty() {
            continue;
        }
        let pref = PREF.iter().position(|l| *l == m.lang).map(|p| -(p as i32));
        let e = counts.entry(g).or_insert((0, pref.unwrap_or(-99)));
        e.0 += 1;
        if let Some(p) = pref {
            e.1 = e.1.max(p);
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, (n, pref))| (*n, *pref))
        .map(|(g, _)| g.to_string())
        .unwrap_or_default()
}

/// Generate the Interslavic word for a cognate set.
pub fn generate_set(set: CognateSet, cfg: &ConsensusConfig, proto_enabled: bool) -> GeneratedWord {
    // One primary source form per language (extra senses become secondary).
    let mut forms: Vec<SourceForm> = Vec::new();
    let mut seen_lang: BTreeMap<&str, bool> = BTreeMap::new();
    for m in &set.members {
        let Some(branch) = branch_of(&m.lang) else {
            continue;
        };
        let latin = normalize::to_phonemic_latin(&m.lang, &m.word);
        let skeleton = ortho::ascii_skeleton(&latin);
        if skeleton.is_empty() {
            continue;
        }
        let first = !seen_lang.contains_key(m.lang.as_str());
        seen_lang.insert(m.lang.as_str(), true);
        forms.push(SourceForm {
            lang_code: m.lang.clone(),
            branch,
            modern: m.lang != "cu",
            norm: NormForm {
                original: m.word.clone(),
                latin,
                skeleton,
                flagged: false,
            },
            source_url: format!(
                "https://en.wiktionary.org/wiki/{}#{}",
                m.word.replace(' ', "_"),
                m.lang
            ),
            primary: first,
        });
    }

    let (forms, reflexive) = consensus::strip_reflexive(forms, set.pos);
    let input = MeaningInput {
        pos: set.pos,
        gender: None,
        gloss: set.gloss.clone(),
        forms,
        is_intl_meaning: false,
        reflexive,
    };

    // Cross-branch consensus surface + alternatives.
    let mut candidates = consensus::generate(&input, cfg);

    // Authoritative form from the *known* Proto-Slavic ancestor.
    let mut reconstruction = None;
    if proto_enabled {
        let reflexes: Vec<String> = input
            .forms
            .iter()
            .filter(|f| f.modern && f.primary)
            .map(|f| f.norm.latin.clone())
            .collect();
        let proto_word = set.proto.trim_start_matches('*');
        let mut pc = crate::proto::generate_with_reflexes(proto_word, set.pos, None, &reflexes);
        if reflexive && !pc.form.is_empty() && !pc.form.ends_with(" sę") {
            pc.form.push_str(" sę");
        }
        if !pc.form.is_empty() {
            pc.trace.insert(
                0,
                crate::model::RuleStep::new(
                    "proto-ancestor",
                    set.proto.clone(),
                    pc.form.clone(),
                    format!(
                        "Praslovjanska rekonstrukcija {} (dana etimologijeju Wiktionary).",
                        set.proto
                    ),
                    Some("https://interslavic.fun/learn/orthography/"),
                ),
            );
            reconstruction = Some(crate::model::Reconstruction {
                word: proto_word.to_string(),
                proto_balto_slavic: String::new(),
                proto_indo_european: String::new(),
                confidence: 1.0,
            });
            // The reconstruction is authoritative for the form; place it first.
            pc.score = 0.99;
            candidates.insert(0, pc);
        }
    }

    // Dedupe by standard spelling, keeping the proto-derived (flavored) form.
    dedupe(&mut candidates);

    // Confidence scales with cognate coverage (the core of the design).
    let n_langs = input
        .forms
        .iter()
        .map(|f| f.lang_code.as_str())
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let mut branches = Vec::new();
    for f in &input.forms {
        if !branches.contains(&f.branch) {
            branches.push(f.branch);
        }
    }
    let n_branches = branches.len();
    let (confidence, score) = coverage_confidence(n_langs, n_branches);
    if let Some(top) = candidates.first_mut() {
        top.confidence = confidence;
        top.score = score;
        top.branch_coverage = n_branches as u8;
    }

    GeneratedWord {
        set,
        candidates,
        confidence,
        score,
        n_langs,
        n_branches,
        reconstruction,
    }
}

/// The confidence model the user asked for: more attesting languages / branches →
/// higher confidence. A single-language guess is Low; a word spread across the
/// branches is High.
fn coverage_confidence(n_langs: usize, n_branches: usize) -> (Confidence, f32) {
    let lang_term = (n_langs.min(8) as f32) / 8.0;
    let branch_term = (n_branches as f32) / 3.0;
    let score = (0.12 + 0.55 * lang_term + 0.33 * branch_term).clamp(0.05, 0.99);
    let confidence = if n_langs >= 5 && n_branches >= 2 {
        Confidence::High
    } else if n_langs >= 3 && n_branches >= 2 {
        Confidence::Medium
    } else {
        Confidence::Low
    };
    (confidence, score)
}

fn dedupe(candidates: &mut Vec<Candidate>) {
    use crate::model::CandidateSource;
    candidates.sort_by(|a, b| {
        b.score.total_cmp(&a.score).then(
            ((b.source == CandidateSource::ProtoSlavicRule) as u8)
                .cmp(&((a.source == CandidateSource::ProtoSlavicRule) as u8)),
        )
    });
    let mut seen: Vec<String> = Vec::new();
    let mut out: Vec<Candidate> = Vec::new();
    for c in candidates.drain(..) {
        let key = ortho::to_standard(&c.form.to_lowercase());
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        out.push(c);
    }
    *candidates = out;
}
