//! Bir setin dağılımı ve kompozisyon hedefleriyle karşılaştırılması.
//!
//! Hedefler `docs/gold-set/composition-v0.json`'dadır; gerekçeleri
//! `docs/gold-set/composition.md`'de.

use std::collections::{BTreeMap, HashMap};
use std::fmt;

use mahrem_core::Label;
use serde::Deserialize;

use crate::record::{Domain, GoldRecord, HardCase, Source};

/// Bir setin dağılımı.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stats {
    /// Belge sayısı.
    pub documents: usize,
    /// Belge başına varlık sayısı → belge sayısı.
    pub entities_per_document: BTreeMap<usize, usize>,
    /// Metin türüne göre belge sayısı.
    pub domains: BTreeMap<Domain, usize>,
    /// Metnin kaynağına göre belge sayısı.
    pub sources: BTreeMap<Source, usize>,
    /// Etikete göre span sayısı.
    pub label_spans: BTreeMap<Label, usize>,
    /// Zor durum türüne göre belge sayısı.
    pub hard_cases: BTreeMap<HardCase, usize>,
    /// En az bir zor durum içeren belge sayısı.
    pub hard_case_documents: usize,
    /// En az bir yüksek riskli span içeren belge sayısı.
    pub high_risk_documents: usize,
    /// `child` işaretli span sayısı.
    pub child_spans: usize,
    /// `uncertain` işaretli span sayısı.
    pub uncertain_spans: usize,
}

impl Stats {
    /// Kayıtlardan dağılımı çıkarır.
    #[must_use]
    pub fn from_records<'a>(records: impl IntoIterator<Item = &'a GoldRecord>) -> Self {
        let mut s = Stats::default();
        for rec in records {
            s.documents += 1;
            *s.entities_per_document.entry(rec.spans.len()).or_default() += 1;
            *s.domains.entry(rec.meta.domain).or_default() += 1;
            *s.sources.entry(rec.meta.source).or_default() += 1;
            for case in &rec.meta.hard_cases {
                *s.hard_cases.entry(*case).or_default() += 1;
            }
            if !rec.meta.hard_cases.is_empty() {
                s.hard_case_documents += 1;
            }
            if rec.spans.iter().any(|sp| sp.label.is_high_risk()) {
                s.high_risk_documents += 1;
            }
            for sp in &rec.spans {
                *s.label_spans.entry(sp.label).or_default() += 1;
                s.child_spans += usize::from(sp.child);
                s.uncertain_spans += usize::from(sp.uncertain);
            }
        }
        s
    }

    /// `[min, max]` varlık sayısına sahip belge sayısı; `max` yoksa üst sınırsız.
    #[must_use]
    pub fn documents_with_entities(&self, min: usize, max: Option<usize>) -> usize {
        self.entities_per_document
            .range(min..=max.unwrap_or(usize::MAX))
            .map(|(_, n)| n)
            .sum()
    }

    /// İnsan için kısa özet: metin türleri, varlık sayısı kovaları,
    /// etiketler, yüksek riskli belge, çocuk span'i, zor durum ve elle yazım
    /// oranı. Her satır `- ` ile başlar. Metin ya da değer içermez.
    #[must_use]
    pub fn summary(&self) -> String {
        use std::fmt::Write as _;
        let pct = |n: usize| self.share(n) * 100.0;
        let mut out = String::new();
        let domains: Vec<String> = self
            .domains
            .iter()
            .map(|(d, n)| format!("{d:?} {:.0}%", pct(*n)))
            .collect();
        let _ = writeln!(out, "- Metin türleri: {}", domains.join(", "));
        let buckets = [
            ("0", self.documents_with_entities(0, Some(0))),
            ("1", self.documents_with_entities(1, Some(1))),
            ("2-3", self.documents_with_entities(2, Some(3))),
            ("4-5", self.documents_with_entities(4, Some(5))),
            ("6+", self.documents_with_entities(6, None)),
        ];
        let b: Vec<String> = buckets
            .iter()
            .map(|(k, n)| format!("{k}: {:.0}%", pct(*n)))
            .collect();
        let _ = writeln!(out, "- Varlık sayısı: {}", b.join(", "));
        let labels: Vec<String> = self
            .label_spans
            .iter()
            .map(|(l, n)| format!("{l} {n}"))
            .collect();
        let _ = writeln!(out, "- Etiketler: {}", labels.join(", "));
        let _ = writeln!(
            out,
            "- Yüksek riskli belge: {}, çocuk span: {}, zor durumlu belge: {:.0}%, elle yazılmış: {:.0}%",
            self.high_risk_documents,
            self.child_spans,
            pct(self.hard_case_documents),
            pct(self.sources.get(&Source::Handwritten).copied().unwrap_or(0))
        );
        out
    }

    fn share(&self, n: usize) -> f64 {
        if self.documents == 0 {
            return 0.0;
        }
        #[allow(clippy::cast_precision_loss)]
        let r = n as f64 / self.documents as f64;
        r
    }
}

/// Kapalı oran aralığı, `[min, max]`.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(try_from = "[f64; 2]")]
pub struct Range {
    /// Alt sınır.
    pub min: f64,
    /// Üst sınır.
    pub max: f64,
}

impl TryFrom<[f64; 2]> for Range {
    type Error = String;

    fn try_from([min, max]: [f64; 2]) -> Result<Self, Self::Error> {
        if (0.0..=1.0).contains(&min) && (0.0..=1.0).contains(&max) && min <= max {
            Ok(Range { min, max })
        } else {
            Err(format!("geçersiz oran aralığı [{min}, {max}]"))
        }
    }
}

impl Range {
    fn contains(self, x: f64) -> bool {
        // Kayan nokta hatası sınırdaki değeri dışarıda bırakmasın.
        x >= self.min - 1e-9 && x <= self.max + 1e-9
    }
}

/// Belge başına varlık sayısı kovası.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityBucket {
    /// En az varlık sayısı.
    pub min: usize,
    /// En çok varlık sayısı; `null` ise üst sınırsız.
    pub max: Option<usize>,
    /// Bu kovadaki belgelerin oranı.
    pub share: Range,
}

/// Bir altın set bölümünün kompozisyon hedefleri.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Composition {
    /// Hedeflerin sürümü (`v0`).
    pub version: String,
    /// Bölüm başına belge sayısı (en az).
    pub documents_per_part: usize,
    /// Belge başına varlık sayısı dağılımı.
    pub entities_per_document: Vec<EntityBucket>,
    /// Metin türü oranları.
    pub domains: HashMap<Domain, Range>,
    /// Her etiket için en az span sayısı.
    pub min_spans_per_label: usize,
    /// En az bir yüksek riskli span içeren en az belge sayısı.
    pub min_high_risk_documents: usize,
    /// En az `child` işaretli span sayısı.
    pub min_child_spans: usize,
    /// En az bir zor durum içeren belgelerin en düşük oranı.
    pub min_hard_case_share: f64,
    /// Elle yazılmış belgelerin en düşük oranı.
    pub min_handwritten_share: f64,
}

/// Hedeften bir sapma.
#[derive(Debug, Clone, PartialEq)]
pub struct Deviation {
    /// Hangi hedef.
    pub target: String,
    /// Ölçülen değer.
    pub actual: String,
    /// Beklenen değer.
    pub expected: String,
}

impl fmt::Display for Deviation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} (beklenen {})",
            self.target, self.actual, self.expected
        )
    }
}

fn pct(x: f64) -> String {
    format!("{:.1}%", x * 100.0)
}

impl Composition {
    /// JSON'dan okur.
    ///
    /// # Errors
    ///
    /// JSON geçersizse ya da bir oran aralığı `[0, 1]` dışındaysa.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Dağılımı hedeflerle karşılaştırır; sapma yoksa boş döner.
    #[must_use]
    pub fn check(&self, s: &Stats) -> Vec<Deviation> {
        let mut out = Vec::new();
        let mut dev = |target: String, actual: String, expected: String| {
            out.push(Deviation {
                target,
                actual,
                expected,
            });
        };
        if s.documents < self.documents_per_part {
            dev(
                "belge sayısı".into(),
                s.documents.to_string(),
                format!("en az {}", self.documents_per_part),
            );
        }
        for b in &self.entities_per_document {
            let share = s.share(s.documents_with_entities(b.min, b.max));
            if !b.share.contains(share) {
                let range = match b.max {
                    Some(max) if max == b.min => b.min.to_string(),
                    Some(max) => format!("{}-{max}", b.min),
                    None => format!("{}+", b.min),
                };
                dev(
                    format!("{range} varlıklı belge oranı"),
                    pct(share),
                    format!("{}-{}", pct(b.share.min), pct(b.share.max)),
                );
            }
        }
        let mut domains: Vec<_> = self.domains.iter().collect();
        domains.sort_by_key(|(d, _)| **d);
        for (domain, range) in domains {
            let share = s.share(s.domains.get(domain).copied().unwrap_or(0));
            if !range.contains(share) {
                dev(
                    format!("{domain:?} oranı"),
                    pct(share),
                    format!("{}-{}", pct(range.min), pct(range.max)),
                );
            }
        }
        for label in Label::ALL {
            let n = s.label_spans.get(label).copied().unwrap_or(0);
            if n < self.min_spans_per_label {
                dev(
                    format!("{label} span sayısı"),
                    n.to_string(),
                    format!("en az {}", self.min_spans_per_label),
                );
            }
        }
        if s.high_risk_documents < self.min_high_risk_documents {
            dev(
                "yüksek riskli belge sayısı".into(),
                s.high_risk_documents.to_string(),
                format!("en az {}", self.min_high_risk_documents),
            );
        }
        if s.child_spans < self.min_child_spans {
            dev(
                "child span sayısı".into(),
                s.child_spans.to_string(),
                format!("en az {}", self.min_child_spans),
            );
        }
        let hard_share = s.share(s.hard_case_documents);
        if hard_share + 1e-9 < self.min_hard_case_share {
            dev(
                "zor durum içeren belge oranı".into(),
                pct(hard_share),
                format!("en az {}", pct(self.min_hard_case_share)),
            );
        }
        let handwritten_share = s.share(s.sources.get(&Source::Handwritten).copied().unwrap_or(0));
        if handwritten_share + 1e-9 < self.min_handwritten_share {
            dev(
                "elle yazılmış belge oranı".into(),
                pct(handwritten_share),
                format!("en az {}", pct(self.min_handwritten_share)),
            );
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const V0: &str = include_str!("../../../docs/gold-set/composition-v0.json");

    #[test]
    fn v0_targets_parse_and_are_consistent() {
        let c = Composition::from_json(V0).expect("composition-v0.json okunamadı");
        assert_eq!(c.version, "v0");
        // Kovalar 0'dan başlar, boşluksuz ve çakışmasız ilerler, sonuncusu üst sınırsızdır.
        let mut next = 0;
        for b in &c.entities_per_document {
            assert_eq!(b.min, next);
            next = b.max.map_or(usize::MAX, |m| m + 1);
        }
        assert_eq!(next, usize::MAX);
        // Aralıkların ortaları yaklaşık 1'e toplanmalı: hedef tutarlı olsun.
        let mid = |r: &Range| f64::midpoint(r.min, r.max);
        let buckets: f64 = c.entities_per_document.iter().map(|b| mid(&b.share)).sum();
        let domains: f64 = c.domains.values().map(mid).sum();
        assert!((buckets - 1.0).abs() < 0.05, "{buckets}");
        assert!((domains - 1.0).abs() < 0.1, "{domains}");
        let min_spans = c.min_spans_per_label * Label::ALL.len();
        assert!(
            min_spans < c.documents_per_part * 3,
            "etiket alt sınırları ulaşılamaz"
        );
    }

    #[test]
    fn invalid_range_is_rejected() {
        let bad = V0.replacen("0.15", "1.5", 1);
        assert!(Composition::from_json(&bad).is_err());
    }

    #[test]
    fn check_reports_deviations() {
        let c = Composition::from_json(V0).expect("okunamadı");
        let deviations = c.check(&Stats::default());
        let targets: Vec<&str> = deviations.iter().map(|d| d.target.as_str()).collect();
        assert!(targets.contains(&"belge sayısı"));
        assert!(targets.contains(&"PERSON span sayısı"));
        assert!(targets.contains(&"yüksek riskli belge sayısı"));
    }
}
