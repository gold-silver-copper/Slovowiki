//! Manual curation / override file.
//!
//! A tiny, dependency-free reader for a TOML-style `[[override]]` list. Overrides
//! are known exceptions the algorithm is not expected to derive (chiefly
//! internationalisms with idiosyncratic adaptation). They are applied on the
//! production site but are **excluded from pure-algorithm accuracy** — the
//! benchmark never consults them — and separately reported.

use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Override {
    pub meaning: String,
    pub official: String,
    pub reason: String,
    pub sources: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct Overrides {
    by_meaning: HashMap<String, Override>,
}

impl Overrides {
    pub fn load(path: &Path) -> Self {
        let Ok(text) = std::fs::read_to_string(path) else {
            return Overrides::default();
        };
        Self::parse(&text)
    }

    pub fn parse(text: &str) -> Self {
        let mut by_meaning = HashMap::new();
        let mut cur: Option<Override> = None;
        let flush = |cur: &mut Option<Override>, map: &mut HashMap<String, Override>| {
            if let Some(o) = cur.take() {
                if !o.meaning.is_empty() && !o.official.is_empty() {
                    map.insert(o.meaning.to_lowercase(), o);
                }
            }
        };
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with("[[") {
                flush(&mut cur, &mut by_meaning);
                cur = Some(Override {
                    meaning: String::new(),
                    official: String::new(),
                    reason: String::new(),
                    sources: Vec::new(),
                });
                continue;
            }
            let Some((key, val)) = line.split_once('=') else {
                continue;
            };
            let key = key.trim();
            let val = val.trim().trim_matches('"').to_string();
            if let Some(o) = cur.as_mut() {
                match key {
                    "meaning" => o.meaning = val,
                    "official" => o.official = val,
                    "reason" => o.reason = val,
                    "sources" => {
                        o.sources = val
                            .trim_matches(|c| c == '[' || c == ']')
                            .split(',')
                            .map(|s| s.trim().trim_matches('"').to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                    _ => {}
                }
            }
        }
        flush(&mut cur, &mut by_meaning);
        Overrides { by_meaning }
    }

    pub fn lookup(&self, meaning: &str) -> Option<&Override> {
        let key = meaning.trim().to_lowercase();
        if let Some(o) = self.by_meaning.get(&key) {
            return Some(o);
        }
        // Match any comma-separated sub-gloss.
        for part in key.split([',', ';']) {
            if let Some(o) = self.by_meaning.get(part.trim()) {
                return Some(o);
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.by_meaning.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_meaning.is_empty()
    }
}
