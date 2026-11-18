//! Elle yazılmış metinlerin doldurulması.
//!
//! Metinler yer tutucuyla yazılır; aynı üreticiler aynı kurallarla doldurur.
//! Girdi satırı:
//!
//! ```json
//! {"id": "hw-001", "domain": "chat", "text": "Kızım {KİŞİ_1}'in {SAĞLIK_1}'ı var.",
//!  "child": ["KİŞİ_1"], "values": {"SAĞLIK_1": "astım"}, "hard_cases": [], "group": "hw-001"}
//! ```
//!
//! `child`, `values`, `hard_cases` ve `group` isteğe bağlıdır. Özel nitelikli
//! veride `values` ile anlam verilmezse rastgele bir değer seçilir.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use mahrem_core::Label;
use mahrem_core::fake::{Rng, special_category};
use mahrem_dataset::{Domain, HardCase, Source};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::check::leak;
use crate::doc::{RecordMeta, record};
use crate::error::io;
use crate::fill::{FillContext, fill};
use crate::spec::Entity;
use crate::template::{clean, fold, parse};
use crate::{Error, Result};

/// Elle yazılmış bir belge.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandDoc {
    /// Kimlik.
    pub id: String,
    /// Metin türü.
    pub domain: Domain,
    /// Yer tutuculu metin.
    pub text: String,
    /// Çocuğa ait yer tutucular.
    #[serde(default)]
    pub child: Vec<String>,
    /// Sabit değerler: yer tutucu → değer.
    #[serde(default)]
    pub values: BTreeMap<String, String>,
    /// Zor durumlar.
    #[serde(default)]
    pub hard_cases: Vec<HardCase>,
    /// Bölme grubu; yoksa kimlik.
    #[serde(default)]
    pub group: Option<String>,
}

fn label_of(key: &str) -> Option<Label> {
    let (name, n) = key.rsplit_once('_')?;
    n.parse::<u32>().ok()?;
    let name = fold(name);
    Label::ALL
        .iter()
        .copied()
        .find(|l| fold(l.placeholder_name()) == name)
}

/// Metindeki yer tutucuların anahtarları, ilk geçiş sırasıyla.
fn keys_in(text: &str) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else { break };
        let k = after[..close].trim().to_owned();
        if !keys.iter().any(|x| fold(x) == fold(&k)) {
            keys.push(k);
        }
        rest = &after[close + 1..];
    }
    keys
}

/// Tek bir belgeyi doldurur.
///
/// # Errors
///
/// Bilinmeyen yer tutucu, yer tutucu dışında kalmış veri, eke uymayan değer
/// ya da doğrulayıcı bulgusu; kısa gerekçe.
pub fn fill_doc(
    doc: &HandDoc,
    seed: u64,
) -> std::result::Result<mahrem_dataset::GoldRecord, String> {
    let text = clean(&doc.text);
    let mut rng = Rng::new(seed);
    let mut entities = Vec::new();
    for key in keys_in(&text) {
        let label = label_of(&key).ok_or_else(|| format!("bilinmeyen-yer-tutucu {key}"))?;
        let preset = doc
            .values
            .iter()
            .find(|(k, _)| fold(k) == fold(&key))
            .map(|(_, v)| v.clone())
            .or_else(|| special_category(label, &mut rng));
        let child = label.allows_child_flag() && doc.child.iter().any(|c| fold(c) == fold(&key));
        entities.push(Entity {
            label,
            key,
            child,
            preset,
        });
    }
    entities.sort_by_key(|e| usize::from(e.label != Label::Person));
    let keys: Vec<String> = entities.iter().map(|e| e.key.clone()).collect();
    let pieces = parse(&text, &keys).map_err(str::to_owned)?;
    if let Some(reason) = leak(&pieces) {
        return Err(reason.into());
    }
    let fc = FillContext {
        domain: doc.domain,
        hard_case: None,
    };
    let segs = fill(&entities, &pieces, fc, &mut rng).map_err(str::to_owned)?;
    record(
        &doc.id,
        &segs,
        RecordMeta {
            domain: doc.domain,
            hard_cases: doc.hard_cases.clone(),
            source: Source::Handwritten,
            generator: None,
            group: doc.group.clone(),
        },
    )
}

/// Girdi dosyasını doldurup çıktıya yazar. Doldurulamayan belgeleri
/// gerekçesiyle döner; onlar çıktıya yazılmaz.
///
/// # Errors
///
/// Dosya okunamaz ya da bir satır JSON değilse.
pub fn fill_file(input: &Path, seed: u64, output: &Path) -> Result<(usize, Vec<(String, String)>)> {
    let text = fs::read_to_string(input).map_err(io(input))?;
    let mut out = String::new();
    let mut failed = Vec::new();
    let mut written = 0;
    for (i, line) in text
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty())
    {
        let doc: HandDoc = serde_json::from_str(line)
            .map_err(|e| Error::Invalid(format!("{}:{}: {e}", input.display(), i + 1)))?;
        let h = Sha256::digest(format!("{seed}\n{}", doc.id).as_bytes());
        let mut first = [0u8; 8];
        first.copy_from_slice(&h[..8]);
        match fill_doc(&doc, u64::from_be_bytes(first)) {
            Ok(rec) => {
                out.push_str(&serde_json::to_string(&rec).unwrap_or_default());
                out.push('\n');
                written += 1;
            }
            Err(reason) => failed.push((doc.id, reason)),
        }
    }
    fs::write(output, out).map_err(io(output))?;
    Ok((written, failed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_a_handwritten_doc() {
        let doc: HandDoc = serde_json::from_str(
            r#"{"id":"hw-1","domain":"chat","text":"Kızım {KİŞİ_1}'in {SAĞLIK_1}'ı var, {TELEFON_1}'dan ara.","child":["KİŞİ_1"],"values":{"SAĞLIK_1":"astım"}}"#,
        )
        .unwrap();
        let rec = fill_doc(&doc, 7).unwrap();
        assert_eq!(rec.spans.len(), 3);
        assert!(rec.spans[0].child);
        assert_eq!(rec.spans[1].text, "astım");
        assert_eq!(rec.meta.source, Source::Handwritten);
        assert!(rec.text.contains("'ı var"));
    }

    #[test]
    fn unknown_placeholder_is_reported() {
        let doc: HandDoc =
            serde_json::from_str(r#"{"id":"hw-2","domain":"chat","text":"Ben {AD_1}."}"#).unwrap();
        assert!(
            fill_doc(&doc, 1)
                .unwrap_err()
                .starts_with("bilinmeyen-yer-tutucu")
        );
    }
}
