//! Candidate-generation orchestrator (production path).
//!
//! Combines every source into one ranked candidate list and computes the
//! official-dictionary match status. The **benchmark** deliberately does not go
//! through here — it calls the consensus engine directly so the official lemma
//! can never leak into a candidate. Here, on the production site, the official
//! form and manual overrides are allowed as extra, clearly-labeled candidates.

use crate::consensus::{self, ConsensusConfig, MeaningInput};
use crate::model::{Candidate, CandidateSource, Confidence, MatchStatus};
use crate::orthography as ortho;
use crate::overrides::Overrides;

pub struct Generation {
    /// Ranked generated candidates (algorithmic; excludes the official form).
    pub candidates: Vec<Candidate>,
    /// Optional official form, shown separately as "officially attested".
    pub official: Option<String>,
    pub match_status: MatchStatus,
    /// True when a manual override supplied the top form.
    pub overridden: bool,
}

impl Generation {
    pub fn top(&self) -> Option<&Candidate> {
        self.candidates.first()
    }
}

/// Generate ranked candidates for one meaning.
///
/// * `official_isv` — the official lemma, used only for status/display.
/// * `proto_word` — an optional Proto-Slavic reconstruction to also derive from.
pub fn generate(
    input: &MeaningInput,
    official_isv: Option<&str>,
    proto_word: Option<&str>,
    cfg: &ConsensusConfig,
    overrides: &Overrides,
) -> Generation {
    let mut candidates = consensus::generate(input, cfg);

    // Add a Proto-Slavic-derived candidate when a reconstruction is available.
    if let Some(pw) = proto_word.filter(|s| !s.trim().is_empty()) {
        let pc = crate::proto::generate(pw, input.pos, input.gender);
        if !pc.form.is_empty() {
            candidates.push(pc);
        }
    }

    // Manual override (excluded from pure-algorithm accuracy; site-only).
    let mut overridden = false;
    if let Some(o) = overrides.lookup(&input.gloss) {
        let mut c = Candidate::new(o.official.clone(), CandidateSource::ManualOverride, 0.99);
        c.confidence = Confidence::High;
        c.warnings.push(format!("Ručna korektura: {}", o.reason));
        candidates.insert(0, c);
        overridden = true;
    }

    dedupe(&mut candidates);
    // Stable rank by score, then shorter edit-friendliness (prefer simpler form).
    candidates.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then(a.form.chars().count().cmp(&b.form.chars().count()))
    });

    // Status is computed from the *generated* top candidate vs the official form.
    let match_status = match official_isv {
        None => MatchStatus::NoOfficialEntry,
        Some(off) => {
            let matched = candidates
                .iter()
                .take(1)
                .any(|c| ortho::normalized_match(&c.form, off));
            if matched {
                MatchStatus::OfficialMatch
            } else {
                MatchStatus::DiffersFromOfficial
            }
        }
    };

    Generation {
        candidates,
        official: official_isv.map(|s| s.to_string()),
        match_status,
        overridden,
    }
}

/// Collapse candidates that normalize to the same standard spelling, keeping the
/// highest-scoring representative and merging their evidence/trace.
fn dedupe(candidates: &mut Vec<Candidate>) {
    let mut seen: Vec<String> = Vec::new();
    let mut out: Vec<Candidate> = Vec::new();
    // Process in descending score so the kept representative is the best one.
    candidates.sort_by(|a, b| b.score.total_cmp(&a.score));
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
