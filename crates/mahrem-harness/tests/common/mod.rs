//! Testlerin ortak parçaları: sahte aşama, geçici dizin, çıktı denetimi.

#![allow(dead_code, unreachable_pub)]

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use mahrem_harness::{Item, Plan, Stage, StageError};
use serde_json::json;

/// Örneklerin bir kısmını reddeden, bir kısmında ilk denemede geçici hata
/// veren, biraz bekleyen sahte aşama. Çıktısı tohumdan belirlenir.
pub struct Dummy {
    pub max_sleep_ms: u64,
}

pub fn rejected(item: &Item) -> bool {
    item.seed.is_multiple_of(7)
}

impl Stage for Dummy {
    type Output = serde_json::Value;

    async fn run(&self, item: &Item, attempt: u32) -> Result<Self::Output, StageError> {
        if self.max_sleep_ms > 0 {
            tokio::time::sleep(Duration::from_millis(item.seed % self.max_sleep_ms)).await;
        }
        if rejected(item) {
            return Err(StageError::reject("yedinin katı"));
        }
        if item.seed.is_multiple_of(5) && attempt == 0 {
            return Err(StageError::retryable("beşin katı, ilk deneme"));
        }
        Ok(json!({
            "text": format!("Örnek {} için sentetik metin.", item.index),
            "pad": "x".repeat(usize::try_from(item.seed % 3000).unwrap()),
        }))
    }
}

pub fn plan(items: u64, shard_size: u64, concurrency: usize) -> Plan {
    Plan::from_toml(&format!(
        "name = \"deneme\"\nseed = \"tohum\"\nitems = {items}\nshard_size = {shard_size}\nconcurrency = {concurrency}\nmax_attempts = 2\n"
    ))
    .unwrap()
}

pub struct TempDir(pub PathBuf);

impl TempDir {
    pub fn new(name: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mahrem-harness-{name}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Shard'lardaki satırlar: kimlik → satır. Çift kimlik ya da yarım satır
/// varsa testi düşürür.
pub fn shard_lines(out: &Path) -> HashMap<String, serde_json::Value> {
    let mut lines = HashMap::new();
    let mut names: Vec<_> = fs::read_dir(out.join("shards"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    names.sort();
    for path in names {
        let text = fs::read_to_string(&path).unwrap();
        assert!(
            text.is_empty() || text.ends_with('\n'),
            "{} yarım satırla bitiyor",
            path.display()
        );
        for line in text.lines() {
            let value: serde_json::Value = serde_json::from_str(line).unwrap();
            let id = value["item"].as_str().unwrap().to_owned();
            assert!(
                lines.insert(id.clone(), value).is_none(),
                "{id} iki kez yazılmış"
            );
        }
    }
    lines
}

/// Bitmiş bir çıktı dizinini denetler: her örnek ya bir kez shard'da ya da
/// reddedilmiş olarak veritabanında; çıktılar tohumdan beklenenle aynı.
pub fn verify_complete(out: &Path, plan: &Plan) {
    let lines = shard_lines(out);
    let conn = rusqlite::Connection::open(out.join("state.sqlite")).unwrap();
    let mut stmt = conn.prepare("SELECT id, status FROM items").unwrap();
    let status: HashMap<String, String> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    let mut expected_done = BTreeSet::new();
    for index in 0..plan.items {
        let item = plan.item(index);
        if rejected(&item) {
            assert_eq!(
                status.get(&item.id).map(String::as_str),
                Some("rejected"),
                "{}",
                item.id
            );
            assert!(
                !lines.contains_key(&item.id),
                "{} reddedildi ama yazılmış",
                item.id
            );
        } else {
            expected_done.insert(item.id.clone());
            assert_eq!(
                status.get(&item.id).map(String::as_str),
                Some("done"),
                "{}",
                item.id
            );
            let line = &lines[&item.id];
            assert_eq!(line["index"], index);
            assert_eq!(line["seed"], item.seed);
            assert_eq!(
                line["output"]["text"],
                format!("Örnek {index} için sentetik metin.")
            );
        }
    }
    let written: BTreeSet<String> = lines.keys().cloned().collect();
    assert_eq!(written, expected_done);
    assert_eq!(status.len() as u64, plan.items);
}
