//! Tekilleştirme: birebir aynı ve çok benzer metinler.
//!
//! Metin, etiketli değerler etiket adıyla değiştirilip küçültülerek
//! karşılaştırılır; böylece yalnızca değerleri farklı iki metin de aynı
//! sayılır. Benzerlik: sözcük 5-gram'larının Jaccard oranı. Karşılaştırma
//! O(n²); altın set boyunda yeterli.

use std::collections::HashSet;
use std::fmt::Write as _;

use mahrem_core::to_lowercase_tr;
use mahrem_dataset::GoldRecord;

/// Bu oranın üstündeki benzerlik tekrar sayılır.
pub const THRESHOLD: f64 = 0.7;

fn normalized_words(rec: &GoldRecord) -> Vec<String> {
    let chars: Vec<char> = rec.text.chars().collect();
    let mut out = String::new();
    let mut pos = 0;
    for sp in &rec.spans {
        out.extend(&chars[pos..sp.start]);
        let _ = write!(out, " <{}> ", sp.label.id());
        pos = sp.end;
    }
    out.extend(&chars[pos..]);
    to_lowercase_tr(&out)
        .split(|c: char| !(c.is_alphanumeric() || c == '<' || c == '>' || c == '_'))
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect()
}

fn shingles(words: &[String]) -> HashSet<String> {
    if words.len() < 5 {
        return std::iter::once(words.join(" ")).collect();
    }
    words.windows(5).map(|w| w.join(" ")).collect()
}

/// İki kaydın benzerliği, `[0, 1]`.
#[must_use]
pub fn similarity(a: &GoldRecord, b: &GoldRecord) -> f64 {
    jaccard(
        &shingles(&normalized_words(a)),
        &shingles(&normalized_words(b)),
    )
}

#[allow(clippy::cast_precision_loss)]
fn jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    let inter = a.intersection(b).count();
    let union = a.len() + b.len() - inter;
    if union == 0 {
        1.0
    } else {
        inter as f64 / union as f64
    }
}

/// Tekrarları ayıklar: `records` sırasıyla ilerler, kendisinden önce kalan
/// bir kayda ya da `against`'teki bir kayda çok benzeyeni atar. Atılanların
/// kimliklerini ve benzediği kaydı döner.
pub fn dedup(records: &mut Vec<GoldRecord>, against: &[GoldRecord]) -> Vec<(String, String)> {
    let fixed: Vec<(String, HashSet<String>)> = against
        .iter()
        .map(|r| (r.id.clone(), shingles(&normalized_words(r))))
        .collect();
    let mut kept: Vec<(String, HashSet<String>)> = Vec::new();
    let mut dropped = Vec::new();
    records.retain(|r| {
        let sh = shingles(&normalized_words(r));
        if let Some((id, _)) = fixed
            .iter()
            .chain(kept.iter())
            .find(|(_, other)| jaccard(&sh, other) >= THRESHOLD)
        {
            dropped.push((r.id.clone(), id.clone()));
            return false;
        }
        kept.push((r.id.clone(), sh));
        true
    });
    dropped
}
