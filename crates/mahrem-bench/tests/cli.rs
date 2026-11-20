//! `mahrem-bench` binary'si: predict → score, gizlilik, validate, stats.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mahrem_dataset::GoldRecord;
use serde_json::Value;

fn bin(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mahrem-bench"))
        .args(args)
        .output()
        .expect("mahrem-bench çalıştırılamadı")
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn path(rel: &str) -> String {
    root().join(rel).to_string_lossy().into_owned()
}

fn tmp(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "mahrem-bench-{name}-{}-{nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn predict_and_score_hard_cases() {
    let dir = tmp("score");
    let gold = path("datasets/hard-cases/hard-cases.jsonl");
    let pred = dir.join("pred.jsonl").to_string_lossy().into_owned();
    let json = dir.join("report.json").to_string_lossy().into_owned();
    let o = bin(&["predict", "--gold", &gold, "--out", &pred]);
    assert!(o.status.success(), "{o:?}");
    assert!(stdout(&o).contains("p95"));

    let o = bin(&[
        "score",
        "--gold",
        &gold,
        "--pred",
        &pred,
        "--scope",
        "structural",
        "--json",
        &json,
        "--misses",
    ]);
    assert!(o.status.success(), "{o:?}");
    let text = stdout(&o);
    let report: Value = serde_json::from_str(&std::fs::read_to_string(&json).unwrap()).unwrap();
    // Kapsamdaki her yapısal span bulunur; yalnızca yazıyla yazılmış iki numara kaçar.
    assert_eq!(report["scope"], "structural");
    assert_eq!(report["total"]["gold"], 27);
    assert_eq!(report["total"]["protected"], 25);
    assert_eq!(report["false_alarm_documents"], 0);
    assert_eq!(
        report["leak_free"].as_u64().unwrap() + 2,
        report["documents_with_pii"].as_u64().unwrap()
    );
    assert_eq!(report["gold"][0]["file"], "hard-cases.jsonl");
    assert!(
        text.contains("hard-0003") && text.contains("hard-0018"),
        "{text}"
    );

    // Gizlilik: ne terminal çıktısında ne JSON'da altın metinden bir değer geçer.
    let json_text = std::fs::read_to_string(&json).unwrap();
    let data = std::fs::read_to_string(&gold).unwrap();
    for line in data.lines().filter(|l| !l.trim().is_empty()) {
        let rec: GoldRecord = serde_json::from_str(line).unwrap();
        for s in rec.spans.iter().filter(|s| s.text.chars().count() >= 5) {
            assert!(!text.contains(&s.text), "terminalde değer: {:?}", s.text);
            assert!(!json_text.contains(&s.text), "JSON'da değer: {:?}", s.text);
        }
    }
    assert!(
        !json_text.contains(root().to_string_lossy().as_ref()),
        "JSON'da tam yol"
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn invalid_predictions_are_refused() {
    let dir = tmp("invalid");
    let pred = dir.join("pred.jsonl");
    std::fs::write(&pred, "{\"id\": \"yok-boyle-belge\", \"spans\": []}\n").unwrap();
    let o = bin(&[
        "score",
        "--gold",
        &path("docs/gold-set/example.jsonl"),
        "--pred",
        &pred.to_string_lossy(),
    ]);
    assert!(!o.status.success());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn score_example_files() {
    let o = bin(&[
        "score",
        "--gold",
        &path("docs/gold-set/example.jsonl"),
        "--pred",
        &path("docs/gold-set/example.predictions.jsonl"),
    ]);
    assert!(o.status.success(), "{o:?}");
    assert!(stdout(&o).contains("Kapsam: all"));
}

#[test]
fn validate_and_stats() {
    let o = bin(&["validate", &path("datasets/hard-cases/hard-cases.jsonl")]);
    assert!(o.status.success(), "{o:?}");
    assert!(stdout(&o).contains("0 hata"));
    let o = bin(&[
        "validate",
        &path("docs/gold-set/example.jsonl"),
        "--pred",
        &path("docs/gold-set/example.predictions.jsonl"),
    ]);
    assert!(o.status.success(), "{o:?}");

    let dir = tmp("validate");
    let bad = dir.join("bad.jsonl");
    std::fs::write(&bad, "{\"id\": \"x\"}\n").unwrap();
    assert!(!bin(&["validate", &bad.to_string_lossy()]).status.success());
    std::fs::remove_dir_all(dir).unwrap();

    let o = bin(&[
        "stats",
        &path("docs/gold-set/example.jsonl"),
        "--composition",
        "v0",
    ]);
    assert!(o.status.success(), "{o:?}");
    let s = stdout(&o);
    assert!(s.contains("Etiketler") && s.contains("sapmalar"), "{s}");
}

#[test]
fn usage_errors() {
    assert!(!bin(&[]).status.success());
    assert!(!bin(&["score", "--bilinmeyen", "x"]).status.success());
    assert!(bin(&["--version"]).status.success());
}
