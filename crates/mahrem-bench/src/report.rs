//! Raporun terminal ve JSON biçimi. İkisi de yalnızca sayı içerir.

use std::fmt::Write as _;

use mahrem_core::Label;
use mahrem_dataset::{LabelCounts, Miss, Report};
use serde_json::{Value, json};

/// Ölçülen dosyalar.
pub(crate) struct Meta {
    pub(crate) tool: Option<String>,
    /// (dosya adı, belge sayısı, kanonik özet)
    pub(crate) gold: Vec<(String, usize, String)>,
    /// (dosya adı, SHA-256)
    pub(crate) predictions: (String, String),
}

fn pct(x: Option<f64>) -> String {
    x.map_or_else(|| "-".to_owned(), |v| format!("{:.1}%", v * 100.0))
}

fn row(name: &str, c: &LabelCounts) -> String {
    format!(
        "{name:<18} {:>6} {:>9} {:>9} {:>9} {:>11} {:>11}\n",
        c.gold,
        pct(c.protection()),
        pct(c.exact_recall()),
        pct(c.exact_precision()),
        pct(c.overlap_recall()),
        pct(c.overlap_precision()),
    )
}

pub(crate) fn text(r: &Report, m: &Meta) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "mahrem-bench {}", env!("CARGO_PKG_VERSION"));
    if let Some(tool) = &m.tool {
        let _ = writeln!(out, "Araç: {tool}");
    }
    for (name, n, digest) in &m.gold {
        let _ = writeln!(out, "Altın set: {name} ({n} belge, özet {})", &digest[..16]);
    }
    let _ = writeln!(
        out,
        "Tahmin: {} (SHA-256 {})",
        m.predictions.0,
        &m.predictions.1[..16]
    );
    let _ = writeln!(out, "Kapsam: {}, eşik: {}\n", r.scope, r.min_score);
    let _ = writeln!(
        out,
        "Sızıntısız belge oranı          {:>7}  ({}/{})",
        pct(r.leak_free_rate()),
        r.leak_free,
        r.documents_with_pii
    );
    let _ = writeln!(
        out,
        "Yüksek riskli sızıntısız belge  {:>7}  ({}/{})",
        pct(r.high_risk_leak_free_rate()),
        r.high_risk_leak_free,
        r.high_risk_documents
    );
    let _ = writeln!(
        out,
        "Yanlış alarm oranı              {:>7}  ({}/{})",
        pct(r.false_alarm_rate()),
        r.false_alarm_documents,
        r.documents_without_pii
    );
    if r.missing_predictions > 0 {
        let _ = writeln!(
            out,
            "Tahmin kaydı olmayan belge: {} (boş sayıldı)",
            r.missing_predictions
        );
    }
    let _ = writeln!(
        out,
        "\n{:<18} {:>6} {:>9} {:>9} {:>9} {:>11} {:>11}",
        "Etiket", "Altın", "Korunan", "Kesin R", "Kesin P", "Örtüşme R", "Örtüşme P"
    );
    for (label, c) in &r.labels {
        if c.gold > 0 || c.predicted > 0 {
            out.push_str(&row(label.id(), c));
        }
    }
    out.push_str(&row("Toplam", &r.total()));
    out
}

pub(crate) fn misses(misses: &[Miss]) -> String {
    let mut out = String::from("\nKaçan ve fazladan bulunan span'ler (metin yazılmaz):\n");
    for m in misses {
        let kind = if m.leaked { "kaçtı" } else { "fazladan" };
        let _ = writeln!(out, "{kind:<9} {} {}..{} {}", m.id, m.start, m.end, m.label);
    }
    out
}

fn counts(c: &LabelCounts) -> Value {
    json!({
        "gold": c.gold,
        "protected": c.protected,
        "protection": c.protection(),
        "exact": c.exact,
        "exact_recall": c.exact_recall(),
        "overlap": c.overlap,
        "overlap_recall": c.overlap_recall(),
        "predicted": c.predicted,
        "predicted_exact": c.predicted_exact,
        "exact_precision": c.exact_precision(),
        "predicted_overlap": c.predicted_overlap,
        "overlap_precision": c.overlap_precision(),
    })
}

pub(crate) fn json(r: &Report, m: &Meta) -> Value {
    let labels: serde_json::Map<String, Value> = r
        .labels
        .iter()
        .map(|(l, c): (&Label, &LabelCounts)| (l.id().to_owned(), counts(c)))
        .collect();
    json!({
        "mahrem_bench_version": env!("CARGO_PKG_VERSION"),
        "tool": m.tool,
        "gold": m.gold.iter().map(|(f, n, d)| json!({"file": f, "documents": n, "digest": d})).collect::<Vec<_>>(),
        "predictions": {"file": m.predictions.0, "sha256": m.predictions.1},
        "scope": r.scope.name(),
        "min_score": r.min_score,
        "documents": r.documents,
        "documents_with_pii": r.documents_with_pii,
        "leak_free": r.leak_free,
        "leak_free_rate": r.leak_free_rate(),
        "high_risk_documents": r.high_risk_documents,
        "high_risk_leak_free": r.high_risk_leak_free,
        "high_risk_leak_free_rate": r.high_risk_leak_free_rate(),
        "documents_without_pii": r.documents_without_pii,
        "false_alarm_documents": r.false_alarm_documents,
        "false_alarm_rate": r.false_alarm_rate(),
        "missing_predictions": r.missing_predictions,
        "labels": labels,
        "total": counts(&r.total()),
    })
}
