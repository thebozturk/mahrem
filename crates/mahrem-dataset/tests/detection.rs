//! Yapısal tanıyıcılar zor örnekler setine karşı: yazıyla yazılmış numaralar
//! (v0.5) dışındaki yapısal span'lerin hepsi aynı konumla bulunur; etiketsiz
//! numaralarda yanlış alarm olmaz.

use std::path::Path;

use mahrem_core::{Label, analyze};
use mahrem_dataset::GoldRecord;

const STRUCTURAL: &[Label] = &[
    Label::Tckn,
    Label::Ykn,
    Label::Vkn,
    Label::Iban,
    Label::CardNumber,
    Label::Phone,
    Label::Email,
];

/// v0.1'in kapsamında mı? Yazıyla yazılmış numara (rakamsız) v0.5'te gelir.
fn in_scope(label: Label, text: &str) -> bool {
    label == Label::Email || text.chars().any(|c| c.is_ascii_digit())
}

#[test]
fn hard_cases_structural() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../datasets/hard-cases/hard-cases.jsonl");
    let data = std::fs::read_to_string(path).expect("zor örnekler seti okunamadı");
    let mut expected_n = 0;
    let mut missed = Vec::new();
    let mut extra = Vec::new();
    for line in data.lines().filter(|l| !l.trim().is_empty()) {
        let rec: GoldRecord = serde_json::from_str(line).expect("kayıt okunamadı");
        let gold: Vec<_> = rec
            .spans
            .iter()
            .filter(|s| STRUCTURAL.contains(&s.label))
            .collect();
        let found = analyze(&rec.text);
        for g in gold.iter().filter(|g| in_scope(g.label, &g.text)) {
            expected_n += 1;
            if !found
                .iter()
                .any(|f| f.label == g.label && f.start == g.start && f.end == g.end)
            {
                missed.push(format!("{} {} {:?}", rec.id, g.label, g.text));
            }
        }
        for f in &found {
            if !gold
                .iter()
                .any(|g| g.label == f.label && g.start == f.start && g.end == f.end)
            {
                extra.push(format!(
                    "{} {} {:?} ({})",
                    rec.id,
                    f.label,
                    f.text(&rec.text),
                    f.score
                ));
            }
        }
    }
    assert!(expected_n >= 25, "{expected_n}");
    assert!(
        missed.is_empty(),
        "kaçanlar ({}/{expected_n}):\n{}",
        missed.len(),
        missed.join("\n")
    );
    assert!(
        extra.is_empty(),
        "fazladan bulunanlar:\n{}",
        extra.join("\n")
    );
}
