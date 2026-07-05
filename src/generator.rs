//! Candidate-generation orchestrator (production path).
//!
//! Wraps the shared [`crate::pipeline`] (consensus + Proto-Slavic-derived form)
//! and adds the site-only concerns: manual overrides and the official-dictionary
//! match status. The **benchmark** never goes through here (it calls the
//! leakage-free pipeline directly) so the official lemma can never leak into a
//! candidate; here, on the site, the official form and overrides are allowed as
//! clearly-labeled extras.

use crate::consensus::{ConsensusConfig, MeaningInput};
use crate::dump::ProtoIndex;
use crate::model::{Candidate, CandidateSource, Confidence, MatchStatus, Reconstruction};
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
    /// The linked Proto-Slavic reconstruction, if any.
    pub reconstruction: Option<Reconstruction>,
}

impl Generation {
    pub fn top(&self) -> Option<&Candidate> {
        self.candidates.first()
    }
}

/// Generate ranked candidates for one meaning.
///
/// * `official_isv` — the official lemma, used only for status/display.
/// * `proto` — the Proto-Slavic index for reconstruction-derived forms.
pub fn generate(
    input: &MeaningInput,
    official_isv: Option<&str>,
    proto: Option<&ProtoIndex>,
    cfg: &ConsensusConfig,
    overrides: &Overrides,
) -> Generation {
    let (mut candidates, reconstruction) = crate::pipeline::generate(input, proto, cfg);

    // Status is computed from the *generated* top candidate vs the official form,
    // before overrides are applied, so overrides never inflate accuracy.
    let match_status = match official_isv {
        None => MatchStatus::NoOfficialEntry,
        Some(off) => {
            if candidates
                .first()
                .map(|c| ortho::normalized_match(&c.form, off))
                .unwrap_or(false)
            {
                MatchStatus::OfficialMatch
            } else {
                MatchStatus::DiffersFromOfficial
            }
        }
    };

    // Manual override (site-only; excluded from pure-algorithm accuracy).
    let mut overridden = false;
    if let Some(o) = overrides.lookup(&input.gloss) {
        let mut c = Candidate::new(o.official.clone(), CandidateSource::ManualOverride, 0.99);
        c.confidence = Confidence::High;
        c.warnings.push(format!("Ručna korektura: {}", o.reason));
        candidates.insert(0, c);
        overridden = true;
    }

    Generation {
        candidates,
        official: official_isv.map(|s| s.to_string()),
        match_status,
        overridden,
        reconstruction,
    }
}
