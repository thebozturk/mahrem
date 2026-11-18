//! `datasets/` altındaki açık veri setleri: hepsi birlikte hatasız ve
//! uyarısız doğrulanır, kapalı bölüm dosyası repoda bulunmaz.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use mahrem_core::Label;
use mahrem_dataset::{GoldRecord, GoldSet, HardCase};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../datasets")
}

fn jsonl_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("datasets/ okunamadı") {
        let path = entry.expect("dizin girdisi okunamadı").path();
        if path.is_dir() {
            jsonl_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "jsonl") {
            out.push(path);
        }
    }
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{} okunamadı: {e}", path.display()))
}

#[test]
fn open_datasets_are_clean() {
    let mut files = Vec::new();
    jsonl_files(&root(), &mut files);
    files.sort();
    assert!(!files.is_empty(), "datasets/ altında JSONL yok");
    let mut set = GoldSet::new();
    let base = root();
    for path in &files {
        let name = path.strip_prefix(&base).unwrap_or(path).to_string_lossy();
        assert!(
            !name.ends_with(".closed.jsonl"),
            "kapalı bölüm repoda: {name}"
        );
        set.add_jsonl(&name, &read(path));
    }
    let issues: Vec<String> = set.issues().iter().map(ToString::to_string).collect();
    assert!(issues.is_empty(), "{issues:#?}");
}

#[test]
fn hard_cases_cover_every_label_and_case() {
    let content = read(&root().join("hard-cases/hard-cases.jsonl"));
    let records: Vec<GoldRecord> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("geçersiz kayıt"))
        .collect();
    let labels: HashSet<Label> = records
        .iter()
        .flat_map(|r| r.spans.iter().map(|s| s.label))
        .collect();
    let cases: HashSet<HardCase> = records
        .iter()
        .flat_map(|r| r.meta.hard_cases.iter().copied())
        .collect();
    let missing_labels: Vec<_> = Label::ALL.iter().filter(|l| !labels.contains(l)).collect();
    let missing_cases: Vec<_> = HardCase::ALL
        .iter()
        .filter(|c| !cases.contains(c))
        .collect();
    assert!(
        missing_labels.is_empty(),
        "örneği olmayan etiketler: {missing_labels:?}"
    );
    assert!(
        missing_cases.is_empty(),
        "örneği olmayan zor durumlar: {missing_cases:?}"
    );
    assert!(
        records.iter().any(|r| r.spans.is_empty()),
        "kişisel veri içermeyen belge yok"
    );
    assert!(
        records.iter().any(|r| r.spans.iter().any(|s| s.child)),
        "çocuk işareti örneği yok"
    );
    assert!(
        records.iter().any(|r| r.spans.iter().any(|s| s.uncertain)),
        "belirsiz span örneği yok"
    );
}
