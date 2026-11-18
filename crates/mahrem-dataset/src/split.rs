//! Açık/kapalı bölme ve kapalı bölümün özeti.
//!
//! İkisi de dış bir araca gerek kalmadan, başka dillerde de aynı sonucu
//! verecek biçimde tanımlıdır. Tanım: `docs/gold-set/composition.md`.

use std::fmt::Write as _;

use sha2::{Digest, Sha256};

use crate::record::GoldRecord;

/// Bir belgenin düştüğü bölüm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Part {
    /// Açık bölüm (CC BY 4.0).
    Open,
    /// Kapalı bölüm; yayınlanmaz.
    Closed,
}

/// Belgenin bölme grubu: `meta.origin.group`, yoksa belge kimliği.
#[must_use]
pub fn group_of(rec: &GoldRecord) -> &str {
    rec.meta.origin.group.as_deref().unwrap_or(&rec.id)
}

/// Bir grubun bölümünü tohumdan belirler.
///
/// `SHA-256(seed + "\n" + group)` özetinin ilk 8 baytı büyük endian bir
/// tamsayı olarak okunur; `2^64`'e oranı `open_share`'den küçükse grup açık
/// bölüme düşer. Aynı grubun bütün belgeleri aynı bölüme düşer.
///
/// ```
/// use mahrem_dataset::{Part, part_of};
/// assert_eq!(part_of("seed", "g1", 0.0), Part::Closed);
/// assert_eq!(part_of("seed", "g1", 1.0), Part::Open);
/// ```
#[must_use]
pub fn part_of(seed: &str, group: &str, open_share: f64) -> Part {
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    hasher.update(b"\n");
    hasher.update(group.as_bytes());
    let hash = hasher.finalize();
    let mut first = [0u8; 8];
    first.copy_from_slice(&hash[..8]);
    // u64 → f64 dönüşümünde alt bitlerin kaybı bölmeyi etkilemez.
    #[allow(clippy::cast_precision_loss)]
    let x = u64::from_be_bytes(first) as f64 / 18_446_744_073_709_551_616.0;
    if x < open_share {
        Part::Open
    } else {
        Part::Closed
    }
}

/// Özeti alınacak kanonik metin: kayıtlar kimliğe (bayt sırası) göre
/// sıralanır; her kayıt anahtarları sıralı, boşluksuz, ASCII dışı karakterleri
/// kaçırmadan JSON olarak yazılır ve LF ile biter.
///
/// Python karşılığı: `json.dumps(r, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n"`.
///
/// # Panics
///
/// Kayıt JSON'a çevrilemezse; [`GoldRecord`] için bu olmaz.
#[must_use]
pub fn canonical_jsonl(records: &[GoldRecord]) -> String {
    let mut sorted: Vec<&GoldRecord> = records.iter().collect();
    sorted.sort_by(|a, b| a.id.as_bytes().cmp(b.id.as_bytes()));
    let mut out = String::new();
    for rec in sorted {
        // `serde_json::Value`'nun nesneleri anahtar sırasıyla tutulur.
        let value = serde_json::to_value(rec).expect("kayıt JSON'a çevrilemedi");
        out.push_str(&value.to_string());
        out.push('\n');
    }
    out
}

/// Kanonik metnin SHA-256 özeti, küçük harfli onaltılık.
#[must_use]
pub fn digest(records: &[GoldRecord]) -> String {
    let hash = Sha256::digest(canonical_jsonl(records).as_bytes());
    hash.iter().fold(String::with_capacity(64), |mut out, b| {
        let _ = write!(out, "{b:02x}");
        out
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{Domain, Meta, Origin, Source};

    fn rec(id: &str, group: Option<&str>) -> GoldRecord {
        GoldRecord {
            id: id.into(),
            text: "Kargom nerede?".into(),
            spans: vec![],
            meta: Meta {
                schema_version: 1,
                guideline_version: "0.2.0".into(),
                source: Source::Handwritten,
                domain: Domain::Chat,
                hard_cases: vec![],
                origin: Origin {
                    generator: None,
                    run_id: None,
                    group: group.map(Into::into),
                },
            },
        }
    }

    #[test]
    fn groups_stay_together() {
        let a = rec("a", Some("tpl-1"));
        let b = rec("b", Some("tpl-1"));
        assert_eq!(group_of(&a), "tpl-1");
        assert_eq!(group_of(&rec("c", None)), "c");
        for seed in ["s1", "s2", "s3"] {
            assert_eq!(
                part_of(seed, group_of(&a), 0.5),
                part_of(seed, group_of(&b), 0.5)
            );
        }
    }

    #[test]
    fn share_is_respected() {
        let open = (0..2000)
            .filter(|i| part_of("seed", &format!("g{i}"), 0.5) == Part::Open)
            .count();
        assert!((900..1100).contains(&open), "{open}");
    }

    #[test]
    fn canonical_form_is_order_independent() {
        let a = rec("b-2", None);
        let b = rec("a-1", None);
        let one = canonical_jsonl(&[a.clone(), b.clone()]);
        assert_eq!(one, canonical_jsonl(&[b, a]));
        assert!(one.starts_with(r#"{"id":"a-1","meta":{"domain":"chat""#));
        assert!(one.contains("Kargom nerede?"));
        assert_eq!(digest(&[]).len(), 64);
    }
}
