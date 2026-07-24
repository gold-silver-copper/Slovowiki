//! Whole-export form-record fingerprint + differ (V15 item 8, adopted from the
//! interslavic-rs release workflow).
//!
//! The export tree sha proves two builds are identical; this module
//! provides the reviewable record-level explanation for non-identity.
//! The finalized `FormRecord` slice handed to `forms::write_api` is rendered
//! as sorted, keyed, one-per-line text using the exact `api/forms` wire
//! serializer. This covers official, generated, derivative, raw-intl, and
//! corpus-only records plus every serialized field. `fnv1a64` reduces the
//! dump to a pinned number checked by every verified default-data export,
//! and `diff-output` turns "the number moved" into the enumerated record
//! diff. Custom or intentionally degraded exports still report their exact
//! fingerprint without being compared to the default-tree pin.
//!
//! CLI: `dump-output [--out FILE]` runs the default full export in a scratch
//! directory, prints the fingerprint, and optionally writes the canonical
//! dump; `diff-output BEFORE AFTER` compares two dumps.

use anyhow::{Context, Result};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Pinned fingerprint of the finalized 569,538-record `api/forms` surface.
/// A deliberate export change updates this only after `dump-output` files
/// from before and after have been compared with `diff-output`.
pub const EXPORT_FORM_FINGERPRINT: u64 = 0x55a9_a38b_ed7a_13fb;

/// FNV-1a, 64-bit. The 32-bit sibling in forms.rs shards keys; this one
/// fingerprints the whole canonical dump, where 32 bits would collide.
pub fn fnv1a64(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// One line per exported form record, sorted — canonical by construction. The
/// leading JSON strings are the map key and stored lemma key; the remaining
/// JSON array is the exact `api/forms` record encoding, including gloss.
pub(crate) fn canonical_records_dump<'a>(
    records: impl IntoIterator<Item = &'a crate::forms::FormRecord>,
) -> Result<String> {
    let mut lines: Vec<String> = Vec::new();
    for record in records {
        lines.push(format!(
            "{}\t{}\t{}",
            serde_json::to_string(&record.key)?,
            serde_json::to_string(&record.lemma_key)?,
            crate::forms::record_json(record),
        ));
    }
    lines.sort_unstable();
    let mut out = String::with_capacity(lines.len() * 48);
    for l in &lines {
        out.push_str(l);
        out.push('\n');
    }
    Ok(out)
}

/// Verify and optionally persist the exact record surface being exported.
pub(crate) fn verify_export_records(
    records: &[crate::forms::FormRecord],
    dump_path: Option<&Path>,
    enforce_default_pin: bool,
) -> Result<()> {
    let dump = canonical_records_dump(records)?;
    if let Some(path) = dump_path {
        std::fs::write(path, &dump)
            .with_context(|| format!("write record dump {}", path.display()))?;
        println!("wrote {} ({} records)", path.display(), records.len());
    }
    let got = fnv1a64(&dump);
    println!("export form-record fingerprint: {got:016x}");
    if enforce_default_pin {
        anyhow::ensure!(
            got == EXPORT_FORM_FINGERPRINT,
            "export form-record fingerprint moved: {EXPORT_FORM_FINGERPRINT:016x} -> {got:016x} \
             ({} records). Generate before/after dumps with `dump-output --out FILE`, \
             enumerate them with `diff-output BEFORE AFTER`, then deliberately update \
             EXPORT_FORM_FINGERPRINT.",
            records.len()
        );
    } else {
        println!("export form-record pin not enforced for custom or unpinned data inputs");
    }
    Ok(())
}

struct ScratchExport(PathBuf);

impl ScratchExport {
    fn create() -> Result<Self> {
        let path =
            std::env::temp_dir().join(format!("slovowiki-dump-output-{}", std::process::id()));
        std::fs::create_dir(&path).with_context(|| {
            format!(
                "create scratch export {} (remove a stale directory if necessary)",
                path.display()
            )
        })?;
        Ok(Self(path))
    }
}

impl Drop for ScratchExport {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            eprintln!(
                "warning: could not remove scratch export {}: {error}",
                self.0.display()
            );
        }
    }
}

/// `dump-output`: run the same full exporter that produces `api/forms`, print
/// its pinned fingerprint, and optionally write the canonical record dump.
pub fn run_dump(out: Option<&Path>) -> Result<()> {
    let lemmas = Path::new(crate::DEFAULT_LEMMA_CACHE);
    anyhow::ensure!(
        lemmas.exists(),
        "{} is missing — run `make extract-lemmas` first",
        lemmas.display()
    );
    let scratch = ScratchExport::create()?;
    crate::site::export_corpus_with_record_dump(
        lemmas,
        Path::new(crate::DEFAULT_OFFICIAL),
        &scratch.0,
        out,
    )
}

/// `diff-output`: enumerate the record-level differences of two dumps.
pub fn run_diff(before: &Path, after: &Path) -> Result<()> {
    let a = std::fs::read_to_string(before)?;
    let b = std::fs::read_to_string(after)?;
    let sa: std::collections::BTreeSet<&str> = a.lines().collect();
    let sb: std::collections::BTreeSet<&str> = b.lines().collect();
    let removed: Vec<&&str> = sa.difference(&sb).collect();
    let added: Vec<&&str> = sb.difference(&sa).collect();
    let mut s = String::new();
    let _ = writeln!(s, "- removed: {} records", removed.len());
    for l in removed.iter().take(50) {
        let _ = writeln!(s, "  - {l}");
    }
    if removed.len() > 50 {
        let _ = writeln!(s, "  … {} more", removed.len() - 50);
    }
    let _ = writeln!(s, "+ added: {} records", added.len());
    for l in added.iter().take(50) {
        let _ = writeln!(s, "  + {l}");
    }
    if added.len() > 50 {
        let _ = writeln!(s, "  … {} more", added.len() - 50);
    }
    print!("{s}");
    println!("fingerprints: {:016x} -> {:016x}", fnv1a64(&a), fnv1a64(&b));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_fingerprint_pin_cannot_be_disabled() {
        assert_ne!(
            EXPORT_FORM_FINGERPRINT, 0,
            "every full export enforces this pin; zero must never disable the gate"
        );
    }

    #[test]
    fn canonical_record_dump_covers_the_complete_forms_wire_record() {
        let record = crate::forms::FormRecord {
            form: "forma".into(),
            key: "form-key".into(),
            lemma: "lemma".into(),
            lemma_key: "lemma-key".into(),
            entry_id: 7,
            pos: "noun",
            analyses: vec!["nom.jd.".into()],
            source: "lemma",
            status: "official",
            probability: Some(0.375),
            gloss: "gloss".into(),
        };
        let dump = canonical_records_dump([&record]).unwrap();
        assert_eq!(
            dump,
            "\"form-key\"\t\"lemma-key\"\t\
             [\"forma\",\"lemma\",7,\"noun\",[\"nom.jd.\"],\"lemma\",\
             \"official\",0.375,\"gloss\"]\n"
        );

        let mut changed_gloss = record.clone();
        changed_gloss.gloss = "changed gloss".into();
        assert_ne!(
            dump,
            canonical_records_dump([&changed_gloss]).unwrap(),
            "a wire-visible gloss change must move the fingerprint"
        );
        verify_export_records(std::slice::from_ref(&record), None, false)
            .expect("custom/unpinned exports report without enforcing the default-tree pin");
        assert!(
            verify_export_records(std::slice::from_ref(&record), None, true).is_err(),
            "the verified default-data path must enforce the pinned whole-export fingerprint"
        );
    }

    #[test]
    fn fnv1a64_matches_reference_vectors() {
        // Published FNV-1a 64 test vectors.
        assert_eq!(fnv1a64(""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a64("a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a64("foobar"), 0x85944171f73967e8);
    }
}
