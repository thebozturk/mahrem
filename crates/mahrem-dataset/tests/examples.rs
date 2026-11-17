//! `docs/gold-set/` altındaki örnek dosyalar şemaya uymalı: hata da uyarı da yok.

use mahrem_dataset::GoldSet;

fn read(name: &str) -> String {
    let path = format!("{}/../../docs/gold-set/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path} okunamadı: {e}"))
}

#[test]
fn example_gold_set_is_clean() {
    let mut set = GoldSet::new();
    set.add_jsonl("example.jsonl", &read("example.jsonl"));
    let issues: Vec<String> = set.issues().iter().map(ToString::to_string).collect();
    assert!(issues.is_empty(), "{issues:#?}");
    assert!(set.len() >= 5);
}

#[test]
fn example_predictions_are_clean() {
    let mut set = GoldSet::new();
    set.add_jsonl("example.jsonl", &read("example.jsonl"));
    let issues: Vec<String> = set
        .check_predictions(
            "example.predictions.jsonl",
            &read("example.predictions.jsonl"),
        )
        .iter()
        .map(ToString::to_string)
        .collect();
    assert!(issues.is_empty(), "{issues:#?}");
}
