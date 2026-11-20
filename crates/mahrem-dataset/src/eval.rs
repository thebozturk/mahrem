//! Bir aracın tahminlerinin altın sete karşı ölçülmesi.
//!
//! Ana metrik sızıntısız belge oranıdır: en az bir kişisel veri içeren
//! belgelerden hiçbir verisi kaçmamış olanların oranı. Bir altın span,
//! boşluk dışındaki her karakteri herhangi bir tahminle (etiketine
//! bakılmadan) kaplanmışsa korunmuş sayılır: yanlış etiketle maskelenen veri
//! de LLM'e gitmez. Tanımlar ve gerekçe: `docs/benchmark.md`,
//! `docs/decisions/0012-benchmark-metrikleri.md`.
//!
//! Sonuç yalnızca sayıdır; metin ya da değer taşımaz.

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::hash::BuildHasher;
use std::str::FromStr;

use mahrem_core::Label;

use crate::prediction::PredictedSpan;
use crate::record::GoldRecord;

/// Ölçülen etiketler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scope {
    /// Bütün etiketler.
    All,
    /// Altı yapısal veri: TCKN/YKN, VKN, IBAN, kart, telefon, e-posta (v0.1).
    Structural,
    /// Yüksek riskli etiketler: kimlik ve finans numaraları ile KVKK m.6.
    HighRisk,
}

/// Yapısal etiketler.
pub const STRUCTURAL: &[Label] = &[
    Label::Tckn,
    Label::Ykn,
    Label::Vkn,
    Label::Iban,
    Label::CardNumber,
    Label::Phone,
    Label::Email,
];

impl Scope {
    /// Etiket bu kapsamda mı?
    #[must_use]
    pub fn includes(self, label: Label) -> bool {
        match self {
            Scope::All => true,
            Scope::Structural => STRUCTURAL.contains(&label),
            Scope::HighRisk => label.is_high_risk(),
        }
    }

    /// Kapsamın adı.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Scope::All => "all",
            Scope::Structural => "structural",
            Scope::HighRisk => "high-risk",
        }
    }
}

impl FromStr for Scope {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "all" => Ok(Scope::All),
            "structural" => Ok(Scope::Structural),
            "high-risk" => Ok(Scope::HighRisk),
            other => Err(format!(
                "bilinmeyen kapsam {other:?}: all, structural, high-risk"
            )),
        }
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Bir etiketin sayıları.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LabelCounts {
    /// Altın span sayısı.
    pub gold: usize,
    /// Korunan (kaçmayan) altın span sayısı; etikete bakılmaz.
    pub protected: usize,
    /// Aynı etiket ve aynı sınırlarla bulunan altın span sayısı.
    pub exact: usize,
    /// Aynı etiketle en az bir karakter örtüşen tahmini olan altın span sayısı.
    pub overlap: usize,
    /// Bu etiketteki tahmin sayısı.
    pub predicted: usize,
    /// Aynı etiket ve aynı sınırlarla bir altın span'e denk gelen tahmin sayısı.
    pub predicted_exact: usize,
    /// Aynı etiketle bir altın span'le örtüşen tahmin sayısı.
    pub predicted_overlap: usize,
}

fn ratio(n: usize, d: usize) -> Option<f64> {
    #[allow(clippy::cast_precision_loss)]
    (d > 0).then(|| n as f64 / d as f64)
}

impl LabelCounts {
    /// Korunma oranı.
    #[must_use]
    pub fn protection(&self) -> Option<f64> {
        ratio(self.protected, self.gold)
    }

    /// Kesin recall.
    #[must_use]
    pub fn exact_recall(&self) -> Option<f64> {
        ratio(self.exact, self.gold)
    }

    /// Kesin precision.
    #[must_use]
    pub fn exact_precision(&self) -> Option<f64> {
        ratio(self.predicted_exact, self.predicted)
    }

    /// Örtüşmeli recall.
    #[must_use]
    pub fn overlap_recall(&self) -> Option<f64> {
        ratio(self.overlap, self.gold)
    }

    /// Örtüşmeli precision.
    #[must_use]
    pub fn overlap_precision(&self) -> Option<f64> {
        ratio(self.predicted_overlap, self.predicted)
    }
}

/// Ölçüm sonucu.
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    /// Kapsam.
    pub scope: Scope,
    /// Bu skorun altındaki tahminler sayılmadı.
    pub min_score: f64,
    /// Belge sayısı.
    pub documents: usize,
    /// Kapsamda en az bir kişisel veri içeren belge sayısı.
    pub documents_with_pii: usize,
    /// Bunlardan hiçbir verisi kaçmayanların sayısı.
    pub leak_free: usize,
    /// Kapsamda en az bir yüksek riskli span içeren belge sayısı.
    pub high_risk_documents: usize,
    /// Bunlardan hiçbir yüksek riskli verisi kaçmayanların sayısı.
    pub high_risk_leak_free: usize,
    /// Kapsamda kişisel veri içermeyen belge sayısı.
    pub documents_without_pii: usize,
    /// Bunlardan kapsamda en az bir tahmin alanların sayısı.
    pub false_alarm_documents: usize,
    /// Tahmin kaydı olmayan belge sayısı (boş tahmin sayıldı).
    pub missing_predictions: usize,
    /// Etiket başına sayılar; yalnızca kapsamdaki etiketler.
    pub labels: BTreeMap<Label, LabelCounts>,
}

impl Report {
    /// Sızıntısız belge oranı.
    #[must_use]
    pub fn leak_free_rate(&self) -> Option<f64> {
        ratio(self.leak_free, self.documents_with_pii)
    }

    /// Yüksek riskli sızıntısız belge oranı.
    #[must_use]
    pub fn high_risk_leak_free_rate(&self) -> Option<f64> {
        ratio(self.high_risk_leak_free, self.high_risk_documents)
    }

    /// Yanlış alarm oranı.
    #[must_use]
    pub fn false_alarm_rate(&self) -> Option<f64> {
        ratio(self.false_alarm_documents, self.documents_without_pii)
    }

    /// Bütün etiketlerin toplamı.
    #[must_use]
    pub fn total(&self) -> LabelCounts {
        self.labels
            .values()
            .fold(LabelCounts::default(), |mut t, c| {
                t.gold += c.gold;
                t.protected += c.protected;
                t.exact += c.exact;
                t.overlap += c.overlap;
                t.predicted += c.predicted;
                t.predicted_exact += c.predicted_exact;
                t.predicted_overlap += c.predicted_overlap;
                t
            })
    }
}

/// Kaçan ya da fazladan bulunan bir span; metin taşımaz.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Miss {
    /// Belge kimliği.
    pub id: String,
    /// Başlangıç.
    pub start: usize,
    /// Bitiş.
    pub end: usize,
    /// Etiket.
    pub label: Label,
    /// `true`: kaçan altın span; `false`: hiçbir altın span'le örtüşmeyen tahmin.
    pub leaked: bool,
}

fn overlaps(a: (usize, usize), b: (usize, usize)) -> bool {
    a.0 < b.1 && b.0 < a.1
}

/// Tahminleri altın sete karşı ölçer.
///
/// `predictions` belge kimliğinden tahminlere eşlemedir; kaydı olmayan belge
/// boş tahmin sayılır. Skoru `min_score`'un altındaki tahminler atılır;
/// skorsuz tahminler sayılır. Kapsam dışı etiketli tahminler de korumaya
/// katkı verir (veri maskelenmiştir), ama precision ve yanlış alarm
/// hesabına girmez.
#[must_use]
pub fn evaluate<S: BuildHasher>(
    gold: &[GoldRecord],
    predictions: &HashMap<String, Vec<PredictedSpan>, S>,
    scope: Scope,
    min_score: f64,
) -> (Report, Vec<Miss>) {
    let mut report = Report {
        scope,
        min_score,
        documents: gold.len(),
        documents_with_pii: 0,
        leak_free: 0,
        high_risk_documents: 0,
        high_risk_leak_free: 0,
        documents_without_pii: 0,
        false_alarm_documents: 0,
        missing_predictions: 0,
        labels: Label::ALL
            .iter()
            .filter(|l| scope.includes(**l))
            .map(|&l| (l, LabelCounts::default()))
            .collect(),
    };
    let mut misses = Vec::new();
    for rec in gold {
        let preds: Vec<&PredictedSpan> = if let Some(p) = predictions.get(&rec.id) {
            p.iter()
                .filter(|p| p.score.is_none_or(|s| s >= min_score))
                .collect()
        } else {
            report.missing_predictions += 1;
            Vec::new()
        };
        evaluate_doc(rec, &preds, &mut report, &mut misses);
    }
    (report, misses)
}

fn evaluate_doc(
    rec: &GoldRecord,
    preds: &[&PredictedSpan],
    report: &mut Report,
    misses: &mut Vec<Miss>,
) {
    let scope = report.scope;
    let chars: Vec<char> = rec.text.chars().collect();
    let mut covered = vec![false; chars.len()];
    for p in preds {
        for c in covered
            .iter_mut()
            .take(p.end.min(chars.len()))
            .skip(p.start)
        {
            *c = true;
        }
    }
    let gold_spans: Vec<_> = rec
        .spans
        .iter()
        .filter(|s| scope.includes(s.label))
        .collect();
    let scoped: Vec<&PredictedSpan> = preds
        .iter()
        .copied()
        .filter(|p| scope.includes(p.label))
        .collect();
    let miss = |start, end, label, leaked| Miss {
        id: rec.id.clone(),
        start,
        end,
        label,
        leaked,
    };

    let mut doc_leak = false;
    let mut high_risk: Option<bool> = None;
    for g in &gold_spans {
        let protected =
            (g.start..g.end.min(chars.len())).all(|i| covered[i] || chars[i].is_whitespace());
        let c = report.labels.entry(g.label).or_default();
        c.gold += 1;
        c.protected += usize::from(protected);
        c.exact += usize::from(scoped.iter().any(|p| same(p, g.label, g.start, g.end)));
        c.overlap += usize::from(
            scoped
                .iter()
                .any(|p| p.label == g.label && overlaps((p.start, p.end), (g.start, g.end))),
        );
        if !protected {
            doc_leak = true;
            misses.push(miss(g.start, g.end, g.label, true));
        }
        if g.label.is_high_risk() {
            high_risk = Some(high_risk.unwrap_or(false) || !protected);
        }
    }
    for p in &scoped {
        let c = report.labels.entry(p.label).or_default();
        c.predicted += 1;
        c.predicted_exact +=
            usize::from(gold_spans.iter().any(|g| same(p, g.label, g.start, g.end)));
        c.predicted_overlap += usize::from(
            gold_spans
                .iter()
                .any(|g| p.label == g.label && overlaps((p.start, p.end), (g.start, g.end))),
        );
        if !rec
            .spans
            .iter()
            .any(|g| overlaps((p.start, p.end), (g.start, g.end)))
        {
            misses.push(miss(p.start, p.end, p.label, false));
        }
    }
    if gold_spans.is_empty() {
        report.documents_without_pii += 1;
        report.false_alarm_documents += usize::from(!scoped.is_empty());
    } else {
        report.documents_with_pii += 1;
        report.leak_free += usize::from(!doc_leak);
    }
    if let Some(leaked) = high_risk {
        report.high_risk_documents += 1;
        report.high_risk_leak_free += usize::from(!leaked);
    }
}

fn same(p: &PredictedSpan, label: Label, start: usize, end: usize) -> bool {
    p.label == label && p.start == start && p.end == end
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{Domain, GoldSpan, Meta, Origin, Source};

    fn span(start: usize, end: usize, label: Label, text: &str) -> GoldSpan {
        GoldSpan {
            start,
            end,
            label,
            text: text.to_owned(),
            uncertain: false,
            child: false,
        }
    }

    fn rec(id: &str, text: &str, spans: Vec<GoldSpan>) -> GoldRecord {
        GoldRecord {
            id: id.into(),
            text: text.into(),
            spans,
            meta: Meta {
                schema_version: 1,
                guideline_version: "0.2.0".into(),
                source: Source::Handwritten,
                domain: Domain::Chat,
                hard_cases: vec![],
                origin: Origin {
                    generator: None,
                    run_id: None,
                    group: None,
                },
            },
        }
    }

    fn pred(start: usize, end: usize, label: Label, score: Option<f64>) -> PredictedSpan {
        PredictedSpan {
            start,
            end,
            label,
            text: None,
            score,
        }
    }

    /// "Ali 0532 123 45 67 10000000146" ve kişisel veri içermeyen bir belge.
    fn gold() -> Vec<GoldRecord> {
        vec![
            rec(
                "a",
                "Ali 0532 123 45 67 10000000146",
                vec![
                    span(0, 3, Label::Person, "Ali"),
                    span(4, 18, Label::Phone, "0532 123 45 67"),
                    span(19, 30, Label::Tckn, "10000000146"),
                ],
            ),
            rec("b", "Kargom nerede?", vec![]),
        ]
    }

    #[test]
    fn wrong_label_still_protects_but_is_not_exact() {
        let preds = HashMap::from([(
            "a".to_owned(),
            vec![
                pred(4, 18, Label::Tckn, None),
                pred(19, 30, Label::Tckn, Some(0.99)),
            ],
        )]);
        let (r, misses) = evaluate(&gold(), &preds, Scope::Structural, 0.5);
        assert_eq!((r.documents_with_pii, r.leak_free), (1, 1));
        let phone = r.labels[&Label::Phone];
        assert_eq!(
            (phone.gold, phone.protected, phone.exact, phone.overlap),
            (1, 1, 0, 0)
        );
        let tckn = r.labels[&Label::Tckn];
        assert_eq!((tckn.predicted, tckn.predicted_exact), (2, 1));
        assert_eq!(r.high_risk_documents, 1);
        assert_eq!(r.high_risk_leak_free, 1);
        assert!(misses.is_empty());
        assert!(!r.labels.contains_key(&Label::Person));
        // Bütün etiketlerle ad kaçtığı için belge sızıntılı.
        let (all, misses) = evaluate(&gold(), &preds, Scope::All, 0.5);
        assert_eq!(all.leak_free, 0);
        assert_eq!(
            misses,
            vec![Miss {
                id: "a".into(),
                start: 0,
                end: 3,
                label: Label::Person,
                leaked: true
            }]
        );
    }

    #[test]
    fn partial_cover_leaks_whitespace_does_not_matter() {
        // Telefonun son iki hanesi açıkta.
        let preds = HashMap::from([(
            "a".to_owned(),
            vec![
                pred(4, 15, Label::Phone, None),
                pred(19, 30, Label::Tckn, None),
            ],
        )]);
        let (r, _) = evaluate(&gold(), &preds, Scope::Structural, 0.5);
        assert_eq!(r.leak_free, 0);
        assert_eq!(r.labels[&Label::Phone].overlap, 1);
        // Gruplar ayrı ayrı maskelense de aradaki boşluklar sızıntı sayılmaz.
        let preds = HashMap::from([(
            "a".to_owned(),
            vec![
                pred(4, 8, Label::Phone, None),
                pred(9, 12, Label::Phone, None),
                pred(13, 15, Label::Phone, None),
                pred(16, 18, Label::Phone, None),
                pred(19, 30, Label::Tckn, None),
            ],
        )]);
        let (r, _) = evaluate(&gold(), &preds, Scope::Structural, 0.5);
        assert_eq!(r.leak_free, 1);
    }

    #[test]
    fn threshold_false_alarms_and_missing_records() {
        let preds = HashMap::from([
            ("a".to_owned(), vec![pred(19, 30, Label::Tckn, Some(0.3))]),
            ("b".to_owned(), vec![pred(0, 6, Label::Phone, Some(0.9))]),
        ]);
        let (r, misses) = evaluate(&gold(), &preds, Scope::Structural, 0.5);
        assert_eq!(r.labels[&Label::Tckn].protected, 0, "eşiğin altı sayılmaz");
        assert_eq!((r.documents_without_pii, r.false_alarm_documents), (1, 1));
        assert_eq!(r.false_alarm_rate(), Some(1.0));
        assert!(misses.iter().any(|m| m.id == "b" && !m.leaked));
        let (r, _) = evaluate(&gold(), &HashMap::new(), Scope::Structural, 0.5);
        assert_eq!(r.missing_predictions, 2);
        assert_eq!(r.leak_free_rate(), Some(0.0));
        // Kapsam dışı etiketli tahmin yanlış alarm sayılmaz.
        let preds = HashMap::from([("b".to_owned(), vec![pred(0, 6, Label::Person, None)])]);
        let (r, _) = evaluate(&gold(), &preds, Scope::Structural, 0.5);
        assert_eq!(r.false_alarm_documents, 0);
    }
}
