//! Proto-Slavic extraction from the raw Wiktextract dump.
//!
//! The dump is ~23 GB, so we stream it exactly once and write a compact cache of
//! the Proto-Slavic (`sla-pro`) reconstructions — their word, glosses, descendant
//! forms, Balto-Slavic / PIE references, and stem class. Everything downstream
//! (linking, the consensus pipeline, the site) reads the cache, never the dump.

use crate::model::Pos;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// One Proto-Slavic reconstruction, distilled from a `sla-pro` page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtoEntry {
    /// The reconstruction with etymological letters intact (yers, nasals, jat).
    pub word: String,
    pub pos: String,
    pub glosses: Vec<String>,
    /// (lang_code, attested form) pairs flattened from the descendant tree.
    pub descendants: Vec<(String, String)>,
    pub pbs: String,
    pub pie: String,
    pub stem_class: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtoCache {
    pub source: String,
    pub entry_count: usize,
    pub entries: Vec<ProtoEntry>,
}

/// Cheap substring prefilter: only fully parse lines whose top-level language is
/// Proto-Slavic. Descendants of a Proto-Slavic page are modern languages, so the
/// exact field `"lang_code": "sla-pro"` is a good selector; we re-verify after
/// parsing.
const MARKER: &str = "\"lang_code\": \"sla-pro\"";

pub fn extract(dump: &Path, out: &Path) -> Result<()> {
    if !dump.exists() {
        anyhow::bail!("dump not found: {}", dump.display());
    }
    let file = File::open(dump).with_context(|| format!("open {}", dump.display()))?;
    let reader = BufReader::with_capacity(8 * 1024 * 1024, file);

    let mut entries: Vec<ProtoEntry> = Vec::new();
    let mut line_count: u64 = 0;
    for line in reader.lines() {
        let line = line?;
        line_count += 1;
        if !line.contains(MARKER) {
            continue;
        }
        let value: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if value.get("lang_code").and_then(Value::as_str) != Some("sla-pro") {
            continue;
        }
        if let Some(entry) = proto_from_value(&value) {
            entries.push(entry);
            if entries.len() % 2000 == 0 {
                eprintln!(
                    "  extracted {} Proto-Slavic entries after {} lines",
                    entries.len(),
                    line_count
                );
            }
        }
    }

    let cache = ProtoCache {
        source: dump.display().to_string(),
        entry_count: entries.len(),
        entries,
    };
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = out.with_extension("json.tmp");
    let mut f = File::create(&tmp)?;
    serde_json::to_writer(&mut f, &cache)?;
    f.flush()?;
    std::fs::rename(&tmp, out)?;
    println!(
        "wrote {} ({} Proto-Slavic entries from {} lines)",
        out.display(),
        cache.entry_count,
        line_count
    );
    Ok(())
}

fn proto_from_value(value: &Value) -> Option<ProtoEntry> {
    let word = value.get("word").and_then(Value::as_str)?.to_string();
    if word.is_empty() {
        return None;
    }
    let pos = Pos::parse(value.get("pos").and_then(Value::as_str).unwrap_or("")).code();

    let mut glosses: Vec<String> = Vec::new();
    if let Some(senses) = value.get("senses").and_then(Value::as_array) {
        for sense in senses {
            if let Some(gs) = sense.get("glosses").and_then(Value::as_array) {
                for g in gs.iter().filter_map(Value::as_str) {
                    let g = g.trim().to_string();
                    if !g.is_empty() && !glosses.contains(&g) {
                        glosses.push(g);
                    }
                }
            }
        }
    }
    glosses.truncate(8);

    let mut descendants: Vec<(String, String)> = Vec::new();
    collect_descendants(
        value.get("descendants").and_then(Value::as_array),
        &mut descendants,
    );
    // Prefer short (lemma-like) forms; cap to keep the cache compact.
    descendants.sort_by_key(|(_, w)| w.split_whitespace().count());
    descendants.truncate(80);

    let (pbs, pie) = proto_refs(value);
    let stem_class = stem_class(value);

    Some(ProtoEntry {
        word,
        pos: pos.to_string(),
        glosses,
        descendants,
        pbs,
        pie,
        stem_class,
    })
}

fn collect_descendants(nodes: Option<&Vec<Value>>, out: &mut Vec<(String, String)>) {
    let Some(nodes) = nodes else { return };
    for node in nodes {
        let code = node.get("lang_code").and_then(Value::as_str).unwrap_or("");
        let word = node.get("word").and_then(Value::as_str).unwrap_or("");
        if !code.is_empty() && !word.is_empty() {
            out.push((code.to_string(), word.to_string()));
        }
        collect_descendants(node.get("descendants").and_then(Value::as_array), out);
    }
}

fn proto_refs(value: &Value) -> (String, String) {
    let mut text = value
        .get("etymology_text")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if let Some(ts) = value.get("etymology_templates").and_then(Value::as_array) {
        for t in ts {
            text.push('\n');
            text.push_str(t.get("expansion").and_then(Value::as_str).unwrap_or(""));
        }
    }
    (
        after_needle(&text, "Proto-Balto-Slavic"),
        after_needle(&text, "Proto-Indo-European"),
    )
}

fn after_needle(text: &str, needle: &str) -> String {
    let Some(idx) = text.find(needle) else {
        return String::new();
    };
    let rest = &text[idx + needle.len()..];
    let Some(star) = rest.find('*') else {
        return String::new();
    };
    rest[star..]
        .split(|c: char| c.is_whitespace() || [',', ';', ']', ')'].contains(&c))
        .next()
        .unwrap_or("")
        .to_string()
}

fn stem_class(value: &Value) -> Option<String> {
    let cats = value.get("categories").and_then(Value::as_array)?;
    for c in cats.iter().filter_map(Value::as_str) {
        let lc = c.to_lowercase();
        for key in [
            "o-stem",
            "a-stem",
            "ā-stem",
            "i-stem",
            "u-stem",
            "n-stem",
            "s-stem",
            "r-stem",
            "jo-stem",
            "ja-stem",
            "consonant stem",
        ] {
            if lc.contains(key) {
                return Some(c.to_string());
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Cache loading + indexes
// ---------------------------------------------------------------------------

/// In-memory Proto-Slavic index used by the linker.
pub struct ProtoIndex {
    pub entries: Vec<ProtoEntry>,
    /// gloss word token -> entry indices.
    by_gloss_token: HashMap<String, Vec<usize>>,
    /// descendant form skeleton -> entry indices (whole-word tokens).
    by_desc_skeleton: HashMap<String, Vec<usize>>,
}

impl ProtoIndex {
    pub fn load(path: &Path) -> Result<Self> {
        let mut json = String::new();
        use std::io::Read;
        File::open(path)
            .with_context(|| format!("open proto cache {}", path.display()))?
            .read_to_string(&mut json)?;
        let cache: ProtoCache = serde_json::from_str(&json).context("parse proto cache")?;
        Ok(Self::build(cache.entries))
    }

    pub fn build(entries: Vec<ProtoEntry>) -> Self {
        let mut by_gloss_token: HashMap<String, Vec<usize>> = HashMap::new();
        let mut by_desc_skeleton: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, e) in entries.iter().enumerate() {
            for g in &e.glosses {
                for tok in gloss_tokens(g) {
                    by_gloss_token.entry(tok).or_default().push(i);
                }
            }
            for (_, form) in &e.descendants {
                for word in form.split_whitespace() {
                    let sk = crate::orthography::ascii_skeleton(word);
                    if sk.len() >= 2 {
                        by_desc_skeleton.entry(sk).or_default().push(i);
                    }
                }
            }
        }
        ProtoIndex {
            entries,
            by_gloss_token,
            by_desc_skeleton,
        }
    }

    pub fn gloss_candidates(&self, gloss: &str) -> Vec<usize> {
        let mut seen = Vec::new();
        for tok in gloss_tokens(gloss) {
            if let Some(v) = self.by_gloss_token.get(&tok) {
                for &i in v {
                    if !seen.contains(&i) {
                        seen.push(i);
                    }
                }
            }
        }
        seen
    }

    pub fn desc_candidates(&self, skeleton: &str) -> Option<&Vec<usize>> {
        self.by_desc_skeleton.get(skeleton)
    }
}

/// Lowercase content-word gloss tokens (drop stopwords and short tokens).
pub fn gloss_tokens(gloss: &str) -> Vec<String> {
    const STOP: &[&str] = &[
        "the", "a", "an", "to", "of", "and", "or", "in", "on", "for", "with", "be", "is", "as",
        "at", "by", "that", "this", "it", "one", "some", "any", "esp", "e", "g",
    ];
    gloss
        .to_lowercase()
        .split(|c: char| !c.is_alphabetic())
        .filter(|t| t.len() >= 3 && !STOP.contains(t))
        .map(|t| t.to_string())
        .collect()
}
