use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use interslavic::{
    Animacy as IsvAnimacy, Case as IsvCase, Gender as IsvGender, Number as IsvNumber,
    Person as IsvPerson, Tense as IsvTense, ISV,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_DUMP: &str = "/Users/kisaczka/Desktop/code/english/raw-wiktextract-data.jsonl";
const DEFAULT_DATA: &str = "data/wiktionary-lab.json";

const MODERN_CORE: &[&str] = &["ru", "uk", "be", "pl", "cs", "sk", "sl", "sh", "bg", "mk"];

#[derive(Parser)]
#[command(
    author,
    version,
    about = "V-pamęti medžuslovjanska slovnikova laboratorija iz Wiktextract JSONL"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build JSON data from a raw Wiktextract JSONL dump. Full scan by default.
    Build {
        #[arg(long, default_value = DEFAULT_DUMP)]
        dump: PathBuf,
        #[arg(long, default_value = DEFAULT_DATA)]
        output: PathBuf,
        /// Development-only cap for Praslovjansky generated entries.
        #[arg(long)]
        max_proto: Option<usize>,
        /// Development-only cap for all Slavic/proto lexemes.
        #[arg(long)]
        max_lexemes: Option<usize>,
    },
    /// Launch a local HTTP server that loads the generated JSON into memory.
    Serve {
        #[arg(long, default_value = DEFAULT_DATA)]
        data: PathBuf,
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        #[arg(long, default_value_t = 8765)]
        port: u16,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Dataset {
    meta: Meta,
    entries: Vec<GeneratedEntry>,
    lexemes: Vec<Lexeme>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Meta {
    dump: String,
    built_at_unix: u64,
    line_count: u64,
    generated_proto_slavic_entries: usize,
    source_lexemes: usize,
    language_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Lexeme {
    id: usize,
    lang: String,
    lang_code: String,
    lemma: String,
    normalized: String,
    pos: String,
    gloss: String,
    etymology: String,
    url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeneratedEntry {
    id: usize,
    generation_kind: String,
    proto_word: String,
    isv_candidate: String,
    normalized: String,
    pos: String,
    gloss: String,
    score: f32,
    coverage: usize,
    core_coverage: usize,
    proto_balto_slavic: String,
    proto_indo_european: String,
    etymology: String,
    source_url: String,
    descendants: Vec<Descendant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Descendant {
    lang: String,
    lang_code: String,
    word: String,
    roman: String,
    sense: String,
    branch: String,
    url: String,
}

#[derive(Debug)]
struct AppState {
    data: Dataset,
    entry_by_id: HashMap<usize, usize>,
    entries_by_candidate: HashMap<String, Vec<usize>>,
    lexemes_by_lang: BTreeMap<String, Vec<usize>>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Build {
            dump,
            output,
            max_proto,
            max_lexemes,
        } => build(&dump, &output, max_proto, max_lexemes),
        Command::Serve { data, host, port } => serve(&data, &host, port),
    }
}

fn slavic_lang_name(code: &str) -> Option<&'static str> {
    match code {
        "ru" => Some("rusky"),
        "uk" => Some("ukrajinsky"),
        "be" => Some("bělorussky"),
        "pl" => Some("poljsky"),
        "cs" => Some("čehsky"),
        "sk" => Some("slovacky"),
        "sl" => Some("slovensky"),
        "sh" => Some("srbsko-horvatsky"),
        "sr" => Some("srbsky"),
        "hr" => Some("horvatsky"),
        "bg" => Some("bȯlgarsky"),
        "mk" => Some("makedonsky"),
        "dsb" => Some("dolnolužičsky"),
        "hsb" => Some("gornolužičsky"),
        "wen" => Some("lužičsky"),
        "csb" => Some("kašubsky"),
        "rue" => Some("rusinsky"),
        "orv" => Some("starovȯzhodnoslovjansky"),
        "cu" => Some("starocŕkovnoslovjansky"),
        "zle-ort" => Some("starorusinsky"),
        "zlw-opl" => Some("staropoljsky"),
        _ => None,
    }
}

fn proto_lang_name(code: &str) -> Option<&'static str> {
    match code {
        "sla-pro" => Some("praslovjansky"),
        "ine-bsl-pro" => Some("prabaltoslavjansky"),
        "ine-pro" => Some("praindoevropejsky"),
        _ => None,
    }
}

fn display_lang_label(code: &str, raw: &str) -> String {
    let base = slavic_lang_name(code).or_else(|| proto_lang_name(code));
    match (base, raw) {
        (Some(base), "Cyrillic script") => format!("{base} (kirilica)"),
        (Some(base), "Latin script") => format!("{base} (latinica)"),
        (Some(base), _) => base.to_string(),
        (None, raw) => raw.to_string(),
    }
}

fn target_lang(code: &str) -> bool {
    slavic_lang_name(code).is_some() || proto_lang_name(code).is_some()
}

fn top_level_lang_code(line: &str) -> Option<&str> {
    // Wiktextract emits top-level `lang_code` near the end of each JSON object;
    // nested descendants/templates can also contain `lang_code`, so use the last
    // occurrence as a cheap prefilter before full JSON parsing.
    let marker = "\"lang_code\": \"";
    let idx = line.rfind(marker)? + marker.len();
    let rest = &line[idx..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

fn build(
    dump: &Path,
    output: &Path,
    max_proto: Option<usize>,
    max_lexemes: Option<usize>,
) -> Result<()> {
    if !dump.exists() {
        bail!("dump not found: {}", dump.display());
    }
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    let file = File::open(dump).with_context(|| format!("open {}", dump.display()))?;
    let reader = BufReader::with_capacity(1024 * 1024, file);

    let mut lexemes = Vec::new();
    let mut entries = Vec::new();
    let mut language_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut line_count = 0u64;

    for line in reader.lines() {
        let line = line?;
        line_count += 1;
        let Some(top_code) = top_level_lang_code(&line) else {
            continue;
        };
        if !target_lang(top_code) {
            continue;
        }
        let value: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let code = value.get("lang_code").and_then(Value::as_str).unwrap_or("");
        if target_lang(code) && max_lexemes.map_or(true, |m| lexemes.len() < m) {
            let id = lexemes.len() + 1;
            let lexeme = lexeme_from_value(id, &value);
            *language_counts
                .entry(format!("{} ({})", lexeme.lang, lexeme.lang_code))
                .or_default() += 1;
            lexemes.push(lexeme);
        }
        if code == "sla-pro" {
            let id = entries.len() + 1;
            entries.push(generated_from_proto(id, &value));
            if entries.len() % 100 == 0 {
                eprintln!(
                    "extracted {} Praslovjansky generated entries, {} source lexemes after {} lines",
                    entries.len(),
                    lexemes.len(),
                    line_count
                );
            }
            if max_proto.map_or(false, |m| entries.len() >= m) {
                break;
            }
        }
    }

    append_slavic_source_entries(&mut entries, &lexemes);

    let meta = Meta {
        dump: dump.display().to_string(),
        built_at_unix: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        line_count,
        generated_proto_slavic_entries: entries.len(),
        source_lexemes: lexemes.len(),
        language_counts,
    };
    let data = Dataset {
        meta,
        entries,
        lexemes,
    };
    let tmp = output.with_extension("json.tmp");
    let mut f = File::create(&tmp)?;
    serde_json::to_writer(&mut f, &data)?;
    f.flush()?;
    fs::rename(&tmp, output)?;
    println!(
        "wrote {} ({} generated entries, {} lexemes)",
        output.display(),
        data.entries.len(),
        data.lexemes.len()
    );
    Ok(())
}

fn lexeme_from_value(id: usize, value: &Value) -> Lexeme {
    let word = str_field(value, "word");
    let raw_lang = str_field(value, "lang");
    let lang_code = str_field(value, "lang_code");
    let lang = display_lang_label(&lang_code, &raw_lang);
    let reconstruction = proto_lang_name(lang_code.as_str()).is_some();
    Lexeme {
        id,
        lang,
        lang_code,
        lemma: word.clone(),
        normalized: normalize_for_search(&word),
        pos: str_field(value, "pos"),
        gloss: first_gloss(value),
        etymology: str_field(value, "etymology_text")
            .chars()
            .take(4000)
            .collect(),
        url: wiktionary_url(
            &word,
            if reconstruction {
                Some(&raw_lang)
            } else {
                None
            },
        ),
    }
}

fn generated_from_proto(id: usize, value: &Value) -> GeneratedEntry {
    let proto_word = str_field(value, "word");
    let descendants = flatten_descendants(value.get("descendants").and_then(Value::as_array), "");
    let (score, coverage, core_coverage) = score_descendants(&descendants);
    let (pbs, pie) = extract_proto_refs(value);
    let isv_candidate = proto_to_isv(&proto_word);
    GeneratedEntry {
        id,
        generation_kind: "proto_slavic".to_string(),
        proto_word: proto_word.clone(),
        normalized: normalize_for_search(&isv_candidate),
        isv_candidate,
        pos: str_field(value, "pos"),
        gloss: first_gloss(value),
        score,
        coverage,
        core_coverage,
        proto_balto_slavic: pbs,
        proto_indo_european: pie,
        etymology: str_field(value, "etymology_text")
            .chars()
            .take(6000)
            .collect(),
        source_url: wiktionary_url(&proto_word, Some("Proto-Slavic")),
        descendants,
    }
}

fn append_slavic_source_entries(entries: &mut Vec<GeneratedEntry>, lexemes: &[Lexeme]) {
    #[derive(Default)]
    struct SourceGroup {
        candidate: String,
        normalized: String,
        pos_counts: HashMap<String, usize>,
        gloss_counts: HashMap<String, usize>,
        lexeme_indices: Vec<usize>,
        languages: BTreeSet<String>,
        core_languages: BTreeSet<String>,
    }

    let mut groups: HashMap<String, SourceGroup> = HashMap::new();
    let existing: BTreeSet<String> = entries
        .iter()
        .map(|entry| entry.normalized.clone())
        .collect();

    for (idx, lexeme) in lexemes.iter().enumerate() {
        if slavic_lang_name(&lexeme.lang_code).is_none() {
            continue;
        }
        let candidate = source_lemma_to_candidate(&lexeme.lemma);
        if candidate.is_empty() {
            continue;
        }
        let normalized = normalize_for_search(&candidate);
        if normalized.is_empty() || existing.contains(&normalized) {
            continue;
        }
        let group = groups
            .entry(normalized.clone())
            .or_insert_with(|| SourceGroup {
                candidate: candidate.clone(),
                normalized: normalized.clone(),
                ..Default::default()
            });
        *group.pos_counts.entry(lexeme.pos.clone()).or_default() += 1;
        if !lexeme.gloss.is_empty() {
            *group.gloss_counts.entry(lexeme.gloss.clone()).or_default() += 1;
        }
        group.languages.insert(lexeme.lang_code.clone());
        if MODERN_CORE.contains(&lexeme.lang_code.as_str()) {
            group.core_languages.insert(lexeme.lang_code.clone());
        }
        if group.lexeme_indices.len() < 80 {
            group.lexeme_indices.push(idx);
        }
    }

    let mut groups: Vec<_> = groups.into_values().collect();
    groups.sort_by(|a, b| a.normalized.cmp(&b.normalized));
    let mut next_id = entries.len() + 1;
    for group in groups {
        let coverage = group.languages.len();
        let core_coverage = group.core_languages.len();
        let score = (0.03 + coverage as f32 / 30.0 + core_coverage as f32 / 20.0).min(0.82);
        let descendants = group
            .lexeme_indices
            .iter()
            .map(|&idx| {
                let lexeme = &lexemes[idx];
                Descendant {
                    lang: lexeme.lang.clone(),
                    lang_code: lexeme.lang_code.clone(),
                    word: lexeme.lemma.clone(),
                    roman: String::new(),
                    sense: lexeme.gloss.clone(),
                    branch: "izvorno slovo".to_string(),
                    url: lexeme.url.clone(),
                }
            })
            .collect::<Vec<_>>();
        let first_source = group
            .lexeme_indices
            .first()
            .and_then(|&idx| lexemes.get(idx));
        entries.push(GeneratedEntry {
            id: next_id,
            generation_kind: "slavic_source".to_string(),
            proto_word: first_source.map(|l| l.lemma.clone()).unwrap_or_else(|| group.candidate.clone()),
            isv_candidate: group.candidate.clone(),
            normalized: group.normalized.clone(),
            pos: most_common(&group.pos_counts).unwrap_or_else(|| "unknown".to_string()),
            gloss: most_common(&group.gloss_counts).unwrap_or_default(),
            score: (score * 1000.0).round() / 1000.0,
            coverage,
            core_coverage,
            proto_balto_slavic: String::new(),
            proto_indo_european: String::new(),
            etymology: "Generovano iz potvrđenih slovjanskyh izvornih form, ibo v izjętyh dannyh ne byla priložena praslovjanska rekonstrukcija za sej kandidat.".to_string(),
            source_url: first_source.map(|l| l.url.clone()).unwrap_or_default(),
            descendants,
        });
        next_id += 1;
    }
}

fn most_common(counts: &HashMap<String, usize>) -> Option<String> {
    counts
        .iter()
        .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0)))
        .map(|(value, _)| value.clone())
}

fn source_lemma_to_candidate(lemma: &str) -> String {
    let trimmed = lemma.trim().trim_start_matches('*');
    let transliterated = transliterate_cyrillic(trimmed);
    let mut out = String::new();
    for ch in transliterated.chars() {
        if ch.is_alphabetic() || matches!(ch, ' ' | '-' | '\'' | '’') {
            out.push(ch);
        }
    }
    out.trim_matches([' ', '-', '\'', '’']).to_lowercase()
}

fn transliterate_cyrillic(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        let repl = match ch {
            'а' | 'А' => "a",
            'б' | 'Б' => "b",
            'в' | 'В' => "v",
            'г' | 'Г' => "g",
            'ґ' | 'Ґ' => "g",
            'д' | 'Д' => "d",
            'е' | 'Е' => "e",
            'ё' | 'Ё' => "jo",
            'є' | 'Є' => "je",
            'ж' | 'Ж' => "ž",
            'з' | 'З' => "z",
            'и' | 'И' => "i",
            'і' | 'І' => "i",
            'ї' | 'Ї' => "ji",
            'й' | 'Й' => "j",
            'к' | 'К' => "k",
            'л' | 'Л' => "l",
            'м' | 'М' => "m",
            'н' | 'Н' => "n",
            'о' | 'О' => "o",
            'п' | 'П' => "p",
            'р' | 'Р' => "r",
            'с' | 'С' => "s",
            'т' | 'Т' => "t",
            'у' | 'У' => "u",
            'ў' | 'Ў' => "u",
            'ф' | 'Ф' => "f",
            'х' | 'Х' => "h",
            'ц' | 'Ц' => "c",
            'ч' | 'Ч' => "č",
            'ш' | 'Ш' => "š",
            'щ' | 'Щ' => "šč",
            'ъ' | 'Ъ' => "",
            'ы' | 'Ы' => "y",
            'ь' | 'Ь' => "",
            'э' | 'Э' => "e",
            'ю' | 'Ю' => "ju",
            'я' | 'Я' => "ja",
            'ѣ' | 'Ѣ' => "ě",
            'ѫ' | 'Ѫ' => "ų",
            'ѧ' | 'Ѧ' => "ę",
            'ј' | 'Ј' => "j",
            'љ' | 'Љ' => "lj",
            'њ' | 'Њ' => "nj",
            'ћ' | 'Ћ' => "ć",
            'ђ' | 'Ђ' => "đ",
            'џ' | 'Џ' => "dž",
            'ѕ' | 'Ѕ' => "dz",
            _ => {
                out.push(ch);
                continue;
            }
        };
        out.push_str(repl);
    }
    out
}

fn str_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

fn first_gloss(value: &Value) -> String {
    let mut glosses = Vec::new();
    if let Some(senses) = value.get("senses").and_then(Value::as_array) {
        for sense in senses {
            if let Some(gs) = sense.get("glosses").and_then(Value::as_array) {
                for g in gs.iter().filter_map(Value::as_str) {
                    if !glosses.iter().any(|x: &String| x == g) {
                        glosses.push(g.to_string());
                    }
                    if glosses.len() >= 3 {
                        return glosses.join("; ");
                    }
                }
            }
        }
    }
    glosses.join("; ")
}

fn flatten_descendants(nodes: Option<&Vec<Value>>, branch: &str) -> Vec<Descendant> {
    let mut out = Vec::new();
    let Some(nodes) = nodes else {
        return out;
    };
    for node in nodes {
        let lang = node.get("lang").and_then(Value::as_str).unwrap_or("");
        let code = node.get("lang_code").and_then(Value::as_str).unwrap_or("");
        let word = node.get("word").and_then(Value::as_str).unwrap_or("");
        let mut next_branch = branch.to_string();
        if !lang.is_empty() && word.is_empty() {
            next_branch = if branch.is_empty() {
                lang.to_string()
            } else {
                format!("{branch} / {lang}")
            };
        }
        if !word.is_empty() && slavic_lang_name(code).is_some() {
            out.push(Descendant {
                lang: display_lang_label(code, lang),
                lang_code: code.to_string(),
                word: word.to_string(),
                roman: node
                    .get("roman")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                sense: node
                    .get("sense")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                branch: branch.to_string(),
                url: wiktionary_url(word, None),
            });
        }
        if let Some(children) = node.get("descendants").and_then(Value::as_array) {
            let child_branch = if !word.is_empty()
                && !lang.is_empty()
                && lang != "Cyrillic script"
                && lang != "Latin script"
            {
                if branch.is_empty() {
                    lang.to_string()
                } else {
                    format!("{branch} / {lang}")
                }
            } else {
                next_branch
            };
            out.extend(flatten_descendants(Some(children), &child_branch));
        }
    }
    out
}

fn score_descendants(desc: &[Descendant]) -> (f32, usize, usize) {
    let językov: BTreeSet<&str> = desc
        .iter()
        .map(|d| d.lang_code.as_str())
        .filter(|c| slavic_lang_name(c).is_some())
        .collect();
    let core: BTreeSet<&str> = językov
        .iter()
        .copied()
        .filter(|c| MODERN_CORE.contains(c))
        .collect();
    let coverage = językov.len();
    let core_coverage = core.len();
    let score = (0.15 + coverage as f32 / 18.0 + core_coverage as f32 / 20.0).min(1.0);
    (((score * 1000.0).round()) / 1000.0, coverage, core_coverage)
}

fn extract_proto_refs(value: &Value) -> (String, String) {
    let mut text = str_field(value, "etymology_text");
    if let Some(ts) = value.get("etymology_templates").and_then(Value::as_array) {
        for t in ts {
            text.push('\n');
            text.push_str(t.get("expansion").and_then(Value::as_str).unwrap_or(""));
        }
    }
    (
        regex_like_after(&text, "Proto-Balto-Slavic"),
        regex_like_after(&text, "Proto-Indo-European"),
    )
}

fn regex_like_after(text: &str, needle: &str) -> String {
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

fn proto_to_isv(proto: &str) -> String {
    let mut s = proto
        .trim()
        .trim_start_matches('*')
        .replace(['`', '́', '̀'], "");
    s = strip_accents_keep_letters(&s).replace('x', "h");
    while s.ends_with('ъ') || s.ends_with('ь') {
        s.pop();
    }
    s = s.replace('ь', "e").replace('ъ', "o").replace('å', "a");
    let cleaned: String = s
        .chars()
        .filter(|c| {
            c.is_alphabetic()
                || matches!(
                    *c,
                    ' ' | '-' | 'č' | 'š' | 'ž' | 'ć' | 'đ' | 'ě' | 'ę' | 'ų'
                )
        })
        .collect();
    let trimmed = cleaned.trim_matches([' ', '-']);
    if trimmed.is_empty() {
        proto.trim_start_matches('*').to_string()
    } else {
        trimmed.to_string()
    }
}

fn strip_accents_keep_letters(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        let replacement = match c {
            'ȍ' | 'ȏ' | 'ȯ' | 'ò' | 'ó' | 'ō' => Some("o"),
            'ȁ' | 'ȃ' | 'á' | 'à' | 'ā' => Some("a"),
            'ȅ' | 'ȇ' | 'è' | 'é' | 'ē' => Some("e"),
            'ȉ' | 'ȋ' | 'í' | 'ì' | 'ī' => Some("i"),
            'ȕ' | 'ȗ' | 'ú' | 'ù' | 'ū' => Some("u"),
            'ý' | 'ỳ' | 'ȳ' => Some("y"),
            'ĺ' => Some("l"),
            'ľ' => Some("lj"),
            'ń' => Some("nj"),
            'ŕ' => Some("r"),
            'ś' => Some("s"),
            'ź' => Some("z"),
            'ť' => Some("t"),
            'ď' => Some("d"),
            _ => None,
        };
        if let Some(r) = replacement {
            out.push_str(r)
        } else {
            out.push(c)
        }
    }
    out
}

fn normalize_for_search(text: &str) -> String {
    strip_accents_keep_letters(text)
        .trim()
        .trim_start_matches('*')
        .to_lowercase()
}

fn wiktionary_url(word: &str, reconstruction_lang: Option<&str>) -> String {
    let title = match reconstruction_lang {
        Some(lang) => format!("Reconstruction:{lang}/{word}"),
        None => word.replace(' ', "_"),
    };
    format!("https://en.wiktionary.org/wiki/{}", percent_encode(&title))
}

fn percent_encode(input: &str) -> String {
    let mut out = String::new();
    for b in input.as_bytes() {
        match *b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' | b':' => {
                out.push(*b as char)
            }
            b' ' => out.push('_'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn serve(data_path: &Path, host: &str, port: u16) -> Result<()> {
    let mut json = String::new();
    File::open(data_path)
        .with_context(|| format!("open {}", data_path.display()))?
        .read_to_string(&mut json)?;
    let data: Dataset = serde_json::from_str(&json).context("parse generated dataset")?;
    let state = Arc::new(AppState::new(data));
    let addr = format!("{host}:{port}");
    let listener = TcpListener::bind(&addr).with_context(|| format!("bind {addr}"))?;
    println!(
        "Loaded {} entries and {} lexemes into memory",
        state.data.entries.len(),
        state.data.lexemes.len()
    );
    println!("Serving http://{addr}");
    for stream in listener.incoming() {
        let stream = stream?;
        let state = Arc::clone(&state);
        thread::spawn(move || {
            if let Err(err) = handle_connection(stream, state) {
                eprintln!("request error: {err:?}");
            }
        });
    }
    Ok(())
}

impl AppState {
    fn new(data: Dataset) -> Self {
        let mut entry_by_id = HashMap::new();
        let mut entries_by_candidate: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, e) in data.entries.iter().enumerate() {
            entry_by_id.insert(e.id, idx);
            entries_by_candidate
                .entry(e.normalized.clone())
                .or_default()
                .push(idx);
        }
        let mut lexemes_by_lang: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (idx, l) in data.lexemes.iter().enumerate() {
            lexemes_by_lang
                .entry(format!("{} ({})", l.lang, l.lang_code))
                .or_default()
                .push(idx);
        }
        Self {
            data,
            entry_by_id,
            entries_by_candidate,
            lexemes_by_lang,
        }
    }
}

fn handle_connection(mut stream: TcpStream, state: Arc<AppState>) -> Result<()> {
    let mut buf = [0u8; 8192];
    let n = stream.read(&mut buf)?;
    if n == 0 {
        return Ok(());
    }
    let request = String::from_utf8_lossy(&buf[..n]);
    let first = request.lines().next().unwrap_or("");
    let path = first.split_whitespace().nth(1).unwrap_or("/");
    let (status, body, content_type) = route(path, &state);
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.as_bytes().len()
    );
    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn route(raw_path: &str, state: &AppState) -> (&'static str, String, &'static str) {
    let (path, query) = raw_path.split_once('?').unwrap_or((raw_path, ""));
    match path {
        "/" => (
            "200 OK",
            page("Medžuslovjansky VikiSlovnik", &home(state, query)),
            "text/html",
        ),
        "/lexemes" => (
            "200 OK",
            page("Izvorne slova", &lexemes_page(state, query)),
            "text/html",
        ),
        "/stats" => (
            "200 OK",
            page("Statistika", &stats_page(state)),
            "text/html",
        ),
        "/api/entries" => ("200 OK", entries_json(state, query), "application/json"),
        "/wiktionary" => (
            "200 OK",
            page("Izvor Wiktionary", &wiktionary_frame_page(query)),
            "text/html",
        ),
        "/static/wiktionary.css" => ("200 OK", WIKTIONARY_CSS.to_string(), "text/css"),
        _ if path.starts_with("/entry/") => {
            let id = path.trim_start_matches("/entry/").parse::<usize>().ok();
            if let Some(id) = id {
                ("200 OK", page("Entry", &entry_page(state, id)), "text/html")
            } else {
                (
                    "404 Not Found",
                    page("Ne najdeno", "<div class='card'>Nepravilny id zapisa</div>"),
                    "text/html",
                )
            }
        }
        _ => (
            "404 Not Found",
            page("Ne najdeno", "<div class='card'>Ne najdeno</div>"),
            "text/html",
        ),
    }
}

fn home(state: &AppState, query: &str) -> String {
    let q = query_param(query, "q");
    let rows = search_entries(state, &q, 120);
    let unique_candidates = state.entries_by_candidate.len();
    let descendant_count: usize = state
        .data
        .entries
        .iter()
        .map(|entry| entry.descendants.len())
        .sum();
    let avg_coverage = if state.data.entries.is_empty() {
        0.0
    } else {
        state
            .data
            .entries
            .iter()
            .map(|entry| entry.coverage as f32)
            .sum::<f32>()
            / state.data.entries.len() as f32
    };
    let word_of_moment = word_of_moment(state);
    let top_językov = top_language_counts(state, 8);
    let pos_stats = part_of_speech_stats(state);
    let list_title = if q.is_empty() {
        "Generovane slova"
    } else {
        "Rezultaty iskanja"
    };
    format!(
        "<section class='home-heading'>
            <h1 class='firstHeading'>Medžuslovjansky VikiSlovnik</h1>
            <form class='hero-search'><input type='search' name='q' value='{}' placeholder='Iskati generovane formy, praslovjanske rekonstrukcije ili anglijske opisy'><button>Iskati</button></form>
          </section>
          <section class='wiki-layout'>
            <article class='wiki-main-list'>
              <h2><span class='mw-headline'>{}</span></h2>
              <p class='muted'>{} generovanyh zapisov v pamęti. Nižša ocěna znači menje dokazov iz slovjanskyh językov, ale zapis vse jedno imaje svoju stranicu.</p>
              {}
            </article>
            <aside class='wiki-sidebar'>
              <div class='portal-box stats-portal'><h3>Statistika</h3>
                <table class='wikitable compact-table'>
                  <tr><th>Generovane zapisy</th><td>{}</td></tr>
                  <tr><th>Unikalne kandidaty</th><td>{}</td></tr>
                  <tr><th>Izvorne slova</th><td>{}</td></tr>
                  <tr><th>Citovane formy</th><td>{}</td></tr>
                  <tr><th>Srědnje pokryťje</th><td>{:.1}</td></tr>
                </table>
              </div>
              <div class='portal-box spotlight-card'><h3>Slovo momenta</h3>{}</div>
              <div class='portal-box'><h3>O projektu</h3><ul class='compact-list'><li>Generovane stranicě iz Wiktextract.</li><li>Praslovjanske i slovjanske izvorne dokazy.</li><li>Formy iz <code>interslavic-rs</code>.</li><li>Lokalny sloj za izvory Wiktionary.</li></ul></div>
              <div class='portal-box'><h3>Najvęće izvorne čęsti</h3>{}</div>
              <div class='portal-box'><h3>Čęsti rěči</h3>{}</div>
              <div class='portal-box'><h3>Povezky</h3><ul class='compact-list'><li><a href='/lexemes'>Izvorne slova</a></li><li><a href='/stats'>Statistika sborky</a></li><li><a href='/api/entries?q=bog'>Priměr JSON API</a></li></ul></div>
            </aside>
          </section>",
        esc(&q),
        list_title,
        compact_number(state.data.entries.len()),
        entry_table(state, &rows),
        compact_number(state.data.entries.len()),
        compact_number(unique_candidates),
        compact_number(state.data.lexemes.len()),
        compact_number(descendant_count),
        avg_coverage,
        word_of_moment,
        top_językov,
        pos_stats,
    )
}

fn compact_number(value: usize) -> String {
    let s = value.to_string();
    let mut out = String::new();
    for (idx, ch) in s.chars().rev().enumerate() {
        if idx > 0 && idx % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

fn word_of_moment(state: &AppState) -> String {
    if state.data.entries.is_empty() {
        return "<p class='muted'>Nijedne generovane zapisy ne sų naložene.</p>".to_string();
    }
    let tick = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() / 60)
        .unwrap_or(0) as usize;
    let idx = tick % state.data.entries.len();
    let entry = &state.data.entries[idx];
    format!(
        "<p class='spotlight-word'><a href='/entry/{}'>{}</a> <span class='badge'>{}</span></p><p><b>Anglijski opis:</b> {}</p><p class='muted'>{}; ocěna {:.3}; {} citovane slovjanske raznovidnosti.</p>{}",
        entry.id,
        esc(&entry.isv_candidate),
        esc(&entry.pos),
        esc(&entry.gloss),
        entry_origin_phrase(entry),
        entry.score,
        entry.coverage,
        mini_forms(entry),
    )
}

fn entry_origin_phrase(entry: &GeneratedEntry) -> String {
    if entry.generation_kind == "proto_slavic" {
        format!(
            "Iz praslovjanskogo <span class='mention'>{}</span>",
            esc(&entry.proto_word)
        )
    } else {
        format!(
            "Iz slovjanskogo izvora <span class='mention'>{}</span>",
            esc(&entry.proto_word)
        )
    }
}

fn mini_forms(entry: &GeneratedEntry) -> String {
    match entry.pos.as_str() {
        "noun" | "proper_noun" => format!(
            "<table class='wikitable mini-forms'><tr><th>Nom. jedn.</th><td>{}</td></tr><tr><th>Gen. jedn.</th><td>{}</td></tr><tr><th>Nom. množ.</th><td>{}</td></tr></table>",
            esc(&ISV::noun(&entry.isv_candidate, IsvCase::Nom, IsvNumber::Singular)),
            esc(&ISV::noun(&entry.isv_candidate, IsvCase::Gen, IsvNumber::Singular)),
            esc(&ISV::noun(&entry.isv_candidate, IsvCase::Nom, IsvNumber::Plural)),
        ),
        "verb" => format!(
            "<table class='wikitable mini-forms'><tr><th>1. jedn. teper.</th><td>{}</td></tr><tr><th>2. jedn. teper.</th><td>{}</td></tr><tr><th>3. množ. teper.</th><td>{}</td></tr></table>",
            esc(&safe_isv_verb(&entry.isv_candidate, IsvPerson::First, IsvNumber::Singular)),
            esc(&safe_isv_verb(&entry.isv_candidate, IsvPerson::Second, IsvNumber::Singular)),
            esc(&safe_isv_verb(&entry.isv_candidate, IsvPerson::Third, IsvNumber::Plural)),
        ),
        "adj" | "adjective" => format!(
            "<table class='wikitable mini-forms'><tr><th>M. nom. jedn.</th><td>{}</td></tr><tr><th>Ž. nom. jedn.</th><td>{}</td></tr><tr><th>Gen. množ.</th><td>{}</td></tr></table>",
            esc(&ISV::adj(&entry.isv_candidate, IsvCase::Nom, IsvNumber::Singular, IsvGender::Masculine, IsvAnimacy::Inanimate)),
            esc(&ISV::adj(&entry.isv_candidate, IsvCase::Nom, IsvNumber::Singular, IsvGender::Feminine, IsvAnimacy::Inanimate)),
            esc(&ISV::adj(&entry.isv_candidate, IsvCase::Gen, IsvNumber::Plural, IsvGender::Masculine, IsvAnimacy::Animate)),
        ),
        _ => "<p class='muted'>Kratky prědgled prěgibanja za tų čęst rěči ješče ne jest.</p>".to_string(),
    }
}

fn top_language_counts(state: &AppState, limit: usize) -> String {
    let mut pairs: Vec<_> = state.data.meta.language_counts.iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(a.1));
    let mut out = String::from(
        "<table class='wikitable compact-table'><tr><th>Język</th><th>Slova</th></tr>",
    );
    for (lang, count) in pairs.into_iter().take(limit) {
        out.push_str(&format!(
            "<tr><td>{}</td><td>{}</td></tr>",
            esc(lang),
            compact_number(*count)
        ));
    }
    out.push_str("</table>");
    out
}

fn part_of_speech_stats(state: &AppState) -> String {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for entry in &state.data.entries {
        *counts.entry(entry.pos.as_str()).or_default() += 1;
    }
    let mut pairs: Vec<_> = counts.into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1));
    let mut out = String::from(
        "<table class='wikitable compact-table'><tr><th>Čęst rěči</th><th>Generovane zapisy</th></tr>",
    );
    for (pos, count) in pairs {
        out.push_str(&format!(
            "<tr><td>{}</td><td>{}</td></tr>",
            esc(pos),
            compact_number(count)
        ));
    }
    out.push_str("</table>");
    out
}

fn search_entries(state: &AppState, q: &str, limit: usize) -> Vec<usize> {
    if q.trim().is_empty() {
        let mut idxs: Vec<_> = (0..state.data.entries.len()).collect();
        idxs.sort_by(|a, b| {
            state.data.entries[*b]
                .score
                .total_cmp(&state.data.entries[*a].score)
                .then(
                    state.data.entries[*b]
                        .coverage
                        .cmp(&state.data.entries[*a].coverage),
                )
        });
        idxs.truncate(limit);
        return idxs;
    }
    let nq = normalize_for_search(q);
    if let Some(exact) = state.entries_by_candidate.get(&nq) {
        return exact.iter().copied().take(limit).collect();
    }
    let q_lower = q.to_lowercase();
    let mut scored: Vec<(usize, i32)> = state
        .data
        .entries
        .iter()
        .enumerate()
        .filter_map(|(idx, e)| {
            let mut score = 0;
            if e.normalized.contains(&nq) {
                score += 20;
            }
            if normalize_for_search(&e.proto_word).contains(&nq) {
                score += 15;
            }
            if e.gloss.to_lowercase().contains(&q_lower) {
                score += 10;
            }
            if score > 0 {
                Some((idx, score))
            } else {
                None
            }
        })
        .collect();
    scored.sort_by(|(ai, ascore), (bi, bscore)| {
        bscore.cmp(ascore).then(
            state.data.entries[*bi]
                .score
                .total_cmp(&state.data.entries[*ai].score),
        )
    });
    scored.into_iter().map(|(idx, _)| idx).take(limit).collect()
}

fn entry_table(state: &AppState, idxs: &[usize]) -> String {
    let mut s = String::from("<table class='wikitable'><tr><th>Medžuslovjansky kandidat</th><th>Praslovjansky</th><th>Čęst rěči</th><th>Anglijski opis</th><th>Ocěna</th><th>Pokryťje</th></tr>");
    for &idx in idxs {
        let e = &state.data.entries[idx];
        s.push_str(&format!("<tr><td><a href='/entry/{}'><b>{}</b></a></td><td>{}</td><td>{}</td><td>{}</td><td class='score'>{:.3}</td><td>{} językov</td></tr>", e.id, esc(&e.isv_candidate), esc(&e.proto_word), esc(&e.pos), esc(&e.gloss), e.score, e.coverage));
    }
    s.push_str("</table>");
    s
}

fn entry_page(state: &AppState, id: usize) -> String {
    let Some(&idx) = state.entry_by_id.get(&id) else {
        return "<div class='notice'>Zapis ne najdeny</div>".to_string();
    };
    let e = &state.data.entries[idx];
    let pos_heading = wiktionary_pos_heading(&e.pos);
    let headword_line = format!(
        "<p class='inflection-head'><span class='Latn'>{}</span> <span class='badge'>{}</span></p>",
        esc(&e.isv_candidate),
        esc(&e.pos)
    );
    let back_path = format!("/entry/{}", e.id);
    let etymology = render_etymology(e, &back_path);
    let descendants = descendants_wikitable(e, &back_path);
    let references = references_block(e, &back_path);
    let inflection = render_inflection_table(e);
    let warning = generated_warning(e);
    format!(
        "<h1 id='firstHeading' class='firstHeading'>{}</h1>
         {}
         <div class='toc' role='navigation' aria-label='Sadržanje'>
           <div class='toc-title'>Sadržanje</div>
           <ol>
             <li><a href='#Medžuslovjansky'>Medžuslovjansky</a>
               <ol>
                 <li><a href='#Etimologija'>Etimologija</a></li>
                 <li><a href='#Izgovor'>Izgovor</a></li>
                 <li><a href='#Čęst rěči'>{}</a></li>
                 <li><a href='#Srodne slova'>Srodne slova</a></li>
                 <li><a href='#Izvory'>Izvory</a></li>
               </ol>
             </li>
           </ol>
         </div>
         <h2><span id='Medžuslovjansky' class='mw-headline'>Medžuslovjansky</span></h2>
         {}
         <h3><span id='Etimologija' class='mw-headline'>Etimologija</span></h3>
         {}
         <h3><span id='Izgovor' class='mw-headline'>Izgovor</span></h3>
         <p class='muted'>Izgovor ješče ne jest generovany.</p>
         <h3><span id='Čęst rěči' class='mw-headline'>{}</span></h3>
         {}
         <p><b>Anglijski opis iz Wiktionary:</b></p><ol><li>{}</li></ol>
         <h4><span id='Prěgibanje' class='mw-headline'>Prěgibanje</span></h4>
         {}
         <h4><span id='Srodne slova' class='mw-headline'>Srodne slova</span></h4>
         {}
         <h3><span id='Izvory' class='mw-headline'>Izvory</span></h3>
         {}
         <div class='footer-note'>To jest lokalna generovana stranica, ktora koristi HTML/CSS klasy v stilu Wiktionary. Ona ne vkladava CSS ili JavaScript iz Wikimedia.</div>",
        esc(&e.isv_candidate),
        warning,
        esc(pos_heading),
        source_summary(e, &back_path),
        etymology,
        esc(pos_heading),
        headword_line,
        esc(&e.gloss),
        inflection,
        descendants,
        references,
    )
}

fn render_inflection_table(e: &GeneratedEntry) -> String {
    match e.pos.as_str() {
        "noun" | "proper_noun" => render_noun_declension(&e.isv_candidate),
        "adj" | "adjective" => render_adjective_declension(&e.isv_candidate),
        "verb" => render_verb_conjugation(&e.isv_candidate),
        _ => "<p class='muted'>Za tų čęst rěči ješče nema generovanoj tablicy prěgibanja.</p>"
            .to_string(),
    }
}

fn render_noun_declension(word: &str) -> String {
    let rows = [
        ("Nominativ", IsvCase::Nom),
        ("Akuzativ", IsvCase::Acc),
        ("Genitiv", IsvCase::Gen),
        ("Lokativ", IsvCase::Loc),
        ("Dativ", IsvCase::Dat),
        ("Instrumental", IsvCase::Ins),
    ];
    let mut out = String::from("<table class='wikitable inflection-table'><tr><th>Padež</th><th>Jednina</th><th>Množina</th></tr>");
    for (label, case) in rows {
        out.push_str(&format!(
            "<tr><th>{}</th><td>{}</td><td>{}</td></tr>",
            label,
            esc(&ISV::noun(word, case, IsvNumber::Singular)),
            esc(&ISV::noun(word, case, IsvNumber::Plural)),
        ));
    }
    out.push_str("</table>");
    out
}

fn render_adjective_declension(word: &str) -> String {
    let rows = [
        ("Nominativ", IsvCase::Nom),
        ("Akuzativ", IsvCase::Acc),
        ("Genitiv", IsvCase::Gen),
        ("Lokativ", IsvCase::Loc),
        ("Dativ", IsvCase::Dat),
        ("Instrumental", IsvCase::Ins),
    ];
    let mut out = String::from("<table class='wikitable inflection-table'><tr><th>Padež</th><th>M. živ. jedn.</th><th>M. neživ. jedn.</th><th>Ž. jedn.</th><th>Sr. jedn.</th><th>Množina</th></tr>");
    for (label, case) in rows {
        out.push_str(&format!(
            "<tr><th>{}</th><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            label,
            esc(&ISV::adj(
                word,
                case,
                IsvNumber::Singular,
                IsvGender::Masculine,
                IsvAnimacy::Animate
            )),
            esc(&ISV::adj(
                word,
                case,
                IsvNumber::Singular,
                IsvGender::Masculine,
                IsvAnimacy::Inanimate
            )),
            esc(&ISV::adj(
                word,
                case,
                IsvNumber::Singular,
                IsvGender::Feminine,
                IsvAnimacy::Inanimate
            )),
            esc(&ISV::adj(
                word,
                case,
                IsvNumber::Singular,
                IsvGender::Neuter,
                IsvAnimacy::Inanimate
            )),
            esc(&ISV::adj(
                word,
                case,
                IsvNumber::Plural,
                IsvGender::Masculine,
                IsvAnimacy::Animate
            )),
        ));
    }
    out.push_str("</table>");
    out
}

fn render_verb_conjugation(word: &str) -> String {
    let rows = [
        ("1. osoba jedniny", IsvPerson::First, IsvNumber::Singular),
        ("2. osoba jedniny", IsvPerson::Second, IsvNumber::Singular),
        ("3. osoba jedniny", IsvPerson::Third, IsvNumber::Singular),
        ("1. osoba množiny", IsvPerson::First, IsvNumber::Plural),
        ("2. osoba množiny", IsvPerson::Second, IsvNumber::Plural),
        ("3. osoba množiny", IsvPerson::Third, IsvNumber::Plural),
    ];
    let mut out = String::from(
        "<table class='wikitable inflection-table'><tr><th>Osoba</th><th>Teperešnje vrěme</th></tr>",
    );
    for (label, person, number) in rows {
        out.push_str(&format!(
            "<tr><th>{}</th><td>{}</td></tr>",
            label,
            esc(&safe_isv_verb(word, person, number)),
        ));
    }
    out.push_str("</table>");
    out
}

fn safe_isv_verb(word: &str, person: IsvPerson, number: IsvNumber) -> String {
    std::panic::catch_unwind(|| {
        ISV::verb(
            word,
            person,
            number,
            IsvGender::Masculine,
            IsvTense::Present,
        )
    })
    .unwrap_or_else(|_| "—".to_string())
}

fn wiktionary_pos_heading(pos: &str) -> &str {
    match pos {
        "noun" => "Imennik",
        "verb" => "Glagol",
        "adj" | "adjective" => "Pridavnik",
        "adv" | "adverb" => "Prislovnik",
        "proper_noun" => "Vlastno imę",
        "suffix" => "Sufiks",
        "prefix" => "Prefiks",
        _ => "Entry",
    }
}

fn generated_warning(e: &GeneratedEntry) -> String {
    format!(
        "<div class='generated-warning'><b>Automatično generovany kandidat.</b> Ocěna uvěrjenosti: <span class='score'>{:.3}</span>. Pokryťje dokazov: {} slovjanskyh raznovidnostij, v tom čislu {} glavnyh sovrěmennyh językov. Traktuj formų i opis kako vrěmenny do prověrky.</div>",
        e.score, e.coverage, e.core_coverage
    )
}

fn source_summary(e: &GeneratedEntry, back: &str) -> String {
    format!(
        "<div class='notice'><b>{}:</b> {}<br><b>Generovany kandidat:</b> <span class='mention'>{}</span></div>",
        if e.generation_kind == "proto_slavic" { "Izvorna rekonstrukcija" } else { "Izvorna forma" },
        wiktionary_link(&e.source_url, &format!("<span class='mention'>{}</span>", esc(&e.proto_word)), back),
        esc(&e.isv_candidate),
    )
}

fn render_etymology(e: &GeneratedEntry, back: &str) -> String {
    let mut parts = Vec::new();
    if e.generation_kind == "proto_slavic" {
        parts.push(format!(
            "Iz praslovjanskogo {}.",
            wiktionary_link(
                &e.source_url,
                &format!("<span class='mention'>{}</span>", esc(&e.proto_word)),
                back
            )
        ));
    } else {
        parts.push(format!(
            "Generovano iz slovjanskej izvornej formy {} bez priloženej praslovjanskoj rekonstrukcije v izjętyh dannyh.",
            wiktionary_link(
                &e.source_url,
                &format!("<span class='mention'>{}</span>", esc(&e.proto_word)),
                back
            )
        ));
    }
    if !e.proto_balto_slavic.is_empty() {
        parts.push(format!(
            "Prabaltoslavjansky dokaz: <span class='mention'>{}</span>.",
            esc(&e.proto_balto_slavic)
        ));
    }
    if !e.proto_indo_european.is_empty() {
        parts.push(format!(
            "Praindoevropejsky dokaz: <span class='mention'>{}</span>.",
            esc(&e.proto_indo_european)
        ));
    }
    parts.push("Pokazana medžuslovjanska forma jest generovana tekućimi lokalnymi pravilami kandidatov praslovjansky → medžuslovjansky.".to_string());
    format!(
        "<p>{}</p><h4><span class='mw-headline'>Izjętok etimologije iz Wiktionary</span></h4><pre>{}</pre>",
        parts.join(" "),
        esc(&e.etymology)
    )
}

fn descendants_wikitable(e: &GeneratedEntry, back: &str) -> String {
    if e.descendants.is_empty() {
        return "<p class='muted'>Za sej praslovjansky zapis ne bylo izjęto slovjanskyh naslědnikov.</p>".to_string();
    }
    let mut desc = String::from("<table class='wikitable'><tr><th>Język</th><th>Termin</th><th>Romanizacija</th><th>Smysl</th><th>Větv</th><th>Izvor</th></tr>");
    for d in &e.descendants {
        desc.push_str(&format!(
            "<tr><td>{}</td><td><span class='mention'>{}</span></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            esc(&d.lang), esc(&d.word), esc(&d.roman), esc(&d.sense), esc(&d.branch), wiktionary_link(&d.url, "Wiktionary", back)
        ));
    }
    desc.push_str("</table>");
    desc
}

fn references_block(e: &GeneratedEntry, back: &str) -> String {
    let mut refs = String::from("<ol class='reference-list'>");
    refs.push_str(&format!(
        "<li>Anglijsky Wiktionary, {}, {}.</li>",
        wiktionary_link(&e.source_url, &esc(&e.proto_word), back),
        if e.generation_kind == "proto_slavic" {
            "izvor za praslovjansku rekonstrukciju i drěvo naslědnikov"
        } else {
            "izvor za slovjansku formų i opis"
        }
    ));
    let mut seen = BTreeSet::new();
    for d in &e.descendants {
        let key = format!("{}:{}", d.lang_code, d.word);
        if seen.insert(key) {
            refs.push_str(&format!(
                "<li>Anglijsky Wiktionary, {} ({}).</li>",
                wiktionary_link(&d.url, &esc(&d.word), back),
                esc(&d.lang)
            ));
        }
        if seen.len() >= 25 {
            refs.push_str("<li class='muted'>Dodatkove izvorne povezky naslědnikov sų pokazane v tablici srodnyh slov vyše.</li>");
            break;
        }
    }
    refs.push_str("</ol>");
    refs
}

fn wiktionary_layer_href(url: &str, back: &str) -> String {
    format!(
        "/wiktionary?url={}&back={}",
        percent_encode(url),
        percent_encode(back)
    )
}

fn wiktionary_link(url: &str, label_html: &str, back: &str) -> String {
    format!(
        "<a class='wiktionary-source-link' href='{}'>{}</a>",
        esc(&wiktionary_layer_href(url, back)),
        label_html
    )
}

fn wiktionary_frame_page(query: &str) -> String {
    let url = query_param(query, "url");
    let back = query_param(query, "back");
    let back = if back.is_empty() {
        "/".to_string()
    } else {
        back
    };
    if !url.starts_with("https://en.wiktionary.org/wiki/") {
        return format!(
            "<div class='notice'><a href='{}'>← Nazad k medžuslovjanskomu</a></div><p>Refusing to embed non-Wiktionary URL: <code>{}</code></p>",
            esc(&back),
            esc(&url)
        );
    }
    format!(
        "<div class='source-shell'><div class='source-toolbar'><a class='back-link' href='{back}'>← Nazad k medžuslovjanskomu zapisu</a><span class='muted'>Anglijsky izvor Wiktionary pokazany vnutri sej stranici</span><a class='external-open' href='{url}'>Otvoriti na Wiktionary ↗</a></div><iframe class='wiktionary-frame' src='{url}' title='Anglijsky izvor Wiktionary'></iframe><p class='muted'>Ako izvor ne pokazuje se, Wikimedia može blokovati vkladanje iframe; koristi povezku za otvorjenje vyše.</p></div>",
        back = esc(&back),
        url = esc(&url)
    )
}

fn lexemes_page(state: &AppState, query: &str) -> String {
    let q = query_param(query, "q");
    let lang = query_param(query, "lang");
    let qn = normalize_for_search(&q);
    let q_lower = q.to_lowercase();
    let mut idxs: Vec<usize> = if !lang.is_empty() {
        state
            .lexemes_by_lang
            .get(&lang)
            .cloned()
            .unwrap_or_default()
    } else {
        (0..state.data.lexemes.len()).collect()
    };
    if !q.is_empty() {
        idxs.retain(|&idx| {
            let l = &state.data.lexemes[idx];
            l.normalized.contains(&qn) || l.gloss.to_lowercase().contains(&q_lower)
        });
    }
    idxs.truncate(500);
    let mut lang_options = String::from("<option value=''>Vsi języky</option>");
    for key in state.lexemes_by_lang.keys() {
        lang_options.push_str(&format!(
            "<option value='{}' {}>{}</option>",
            esc(key),
            if *key == lang { "selected" } else { "" },
            esc(key)
        ));
    }
    let back_path = if query.is_empty() {
        "/lexemes".to_string()
    } else {
        format!("/lexemes?{}", query)
    };
    let mut table = String::from(
        "<table class='wikitable'><tr><th>Język</th><th>Zaglavno slovo</th><th>Čęst rěči</th><th>Anglijski opis</th><th>Izvor</th></tr>",
    );
    for idx in idxs {
        let l = &state.data.lexemes[idx];
        table.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            esc(&l.lang),
            esc(&l.lemma),
            esc(&l.pos),
            esc(&l.gloss),
            wiktionary_link(&l.url, "Wiktionary", &back_path)
        ));
    }
    table.push_str("</table>");
    format!("<div class='card'><form><input type='search' name='q' value='{}' placeholder='Iskati izvorne slova'><select name='lang'>{}</select><button>Iskati</button></form></div><div class='card'><h2>Izvorne slova</h2>{}</div>", esc(&q), lang_options, table)
}

fn stats_page(state: &AppState) -> String {
    let mut rows = String::new();
    for (lang, count) in &state.data.meta.language_counts {
        rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td></tr>",
            esc(lang),
            count
        ));
    }
    format!("<div class='card'><h2>Statistika sborky</h2><p>Odval: {}</p><p>Prěgledane linije: {}</p><p>Stvorjeno v času Unix: {}</p></div><div class='card'><h3>Čisla po językah</h3><table class='wikitable'><tr><th>Język</th><th>Slova</th></tr>{}</table></div>", esc(&state.data.meta.dump), state.data.meta.line_count, state.data.meta.built_at_unix, rows)
}

fn entries_json(state: &AppState, query: &str) -> String {
    let q = query_param(query, "q");
    let rows = search_entries(state, &q, 200);
    let values: Vec<&GeneratedEntry> = rows
        .into_iter()
        .map(|idx| &state.data.entries[idx])
        .collect();
    serde_json::to_string(&values).unwrap_or_else(|_| "[]".to_string())
}

fn page(title: &str, body: &str) -> String {
    format!(
        "<!doctype html><html><head><meta charset='utf-8'><meta name='viewport' content='width=device-width, initial-scale=1'><title>{}</title><link rel='stylesheet' href='/static/wiktionary.css'></head><body><div class='vector-page'><header class='vector-header'><h1><a href='/'>Medžuslovjansky VikiSlovnik</a></h1><span class='tagline'>lokalny generovany slovnik v stilu Wiktionary</span></header><main class='mw-body'><div class='mw-body-content'><div class='mw-parser-output'>{}</div></div></main></div></body></html>",
        esc(title),
        body
    )
}

const WIKTIONARY_CSS: &str = include_str!("../static/wiktionary.css");

fn query_param(query: &str, key: &str) -> String {
    for part in query.split('&') {
        let (k, v) = part.split_once('=').unwrap_or((part, ""));
        if k == key {
            return percent_decode(v);
        }
    }
    String::new()
}

fn percent_decode(input: &str) -> String {
    let input = input.replace('+', " ");
    let bytes = input.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = u8::from_str_radix(&input[i + 1..i + 3], 16) {
                out.push(hex);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn esc(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
