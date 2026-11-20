//! Yapısal tanıyıcılar: TCKN/YKN, VKN, IBAN, kart, telefon ve e-posta.
//!
//! Metin bir kez taranır. Rakam grupları izinli ayraçlarla (boşluk, tire,
//! nokta, eğik çizgi, parantez, tek satır sonu) birleşerek aday dizi olur;
//! dizinin içinde grup sınırlarında başlayıp biten her pencere tanıyıcılara
//! sorulur: uzunluk, önek ve checksum. E-posta ayrıca taranır; "at" ve
//! "nokta" ile yazılmış biçim de tanınır. Gerekçe: `docs/decisions/0011-yapisal-tarayici.md`.
//!
//! Konumlar giriş metni üzerinde Unicode code point cinsinden, bitiş
//! hariçtir (Python `str` indeksleriyle aynı).
//!
//! ```
//! use mahrem_core::{Label, analyze};
//!
//! let found = analyze("TCKN: 10000000146, tel 0532 123 45 67");
//! assert_eq!(found[0].label, Label::Tckn);
//! assert_eq!((found[0].start, found[0].end), (6, 17));
//! assert_eq!(found[1].label, Label::Phone);
//! ```

mod context;
mod email;
mod numbers;

use crate::Label;

/// Varsayılan en düşük skor. Bağlamsız VKN (0,4) bunun altında kalır.
pub const DEFAULT_MIN_SCORE: f32 = 0.5;

/// Bulunan bir varlık.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Entity {
    /// Etiket.
    pub label: Label,
    /// Başlangıç (code point, dahil).
    pub start: usize,
    /// Bitiş (code point, hariç).
    pub end: usize,
    /// Güven skoru, `[0, 1]`.
    pub score: f32,
    /// Tanıyıcının adı (`tckn`, `iban`, `phone`, `email-obfuscated` …).
    pub source: &'static str,
}

impl Entity {
    /// Varlığın metindeki karşılığı.
    #[must_use]
    pub fn text<'a>(&self, text: &'a str) -> &'a str {
        let mut indices = text
            .char_indices()
            .map(|(i, _)| i)
            .chain(std::iter::once(text.len()));
        let start = indices.nth(self.start).unwrap_or(text.len());
        let end = if self.end > self.start {
            indices.nth(self.end - self.start - 1).unwrap_or(text.len())
        } else {
            start
        };
        &text[start..end]
    }
}

/// Aday eşleşme; çakışma çözülmeden önce.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Candidate {
    pub(crate) entity: Entity,
    /// Eşit uzunluk ve skorda öncelik; küçük olan önce.
    pub(crate) priority: u8,
}

/// Metindeki yapısal kişisel verileri varsayılan eşikle bulur.
#[must_use]
pub fn analyze(text: &str) -> Vec<Entity> {
    analyze_with(text, DEFAULT_MIN_SCORE)
}

/// Metindeki yapısal kişisel verileri `min_score` eşiğiyle bulur.
///
/// Sonuç başlangıca göre sıralıdır ve çakışma içermez. Çakışan adaylardan
/// en uzun olan, eşitse en yüksek skorlu olan kalır.
#[must_use]
pub fn analyze_with(text: &str, min_score: f32) -> Vec<Entity> {
    let chars: Vec<char> = text.chars().collect();
    let mut found = numbers::scan(&chars);
    found.extend(email::scan(&chars));
    found.retain(|c| c.entity.score >= min_score);
    resolve(found)
}

fn resolve(mut found: Vec<Candidate>) -> Vec<Entity> {
    found.sort_by(|a, b| {
        let (ea, eb) = (&a.entity, &b.entity);
        (eb.end - eb.start)
            .cmp(&(ea.end - ea.start))
            .then(eb.score.total_cmp(&ea.score))
            .then(a.priority.cmp(&b.priority))
            .then(ea.start.cmp(&eb.start))
    });
    let mut kept: Vec<Entity> = Vec::new();
    for c in found {
        let e = c.entity;
        if kept.iter().all(|k| e.end <= k.start || e.start >= k.end) {
            kept.push(e);
        }
    }
    kept.sort_by_key(|e| e.start);
    kept
}

#[cfg(test)]
mod tests;
