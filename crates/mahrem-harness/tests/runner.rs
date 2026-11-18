//! Çalıştırıcının uçtan uca davranışı.

mod common;

use std::fs;
use std::io::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};

use common::{Dummy, TempDir, plan, rejected, shard_lines, verify_complete};
use mahrem_harness::{CancellationToken, HarnessError, Item, Runner, Stage, StageError};
use serde_json::json;

#[tokio::test]
async fn runs_to_completion_and_resumes_as_noop() {
    let dir = TempDir::new("complete");
    let p = plan(120, 25, 6);
    let summary = Runner::new(p.clone(), &dir.0)
        .provenance(json!({"model": "sahte"}))
        .run(Dummy { max_sleep_ms: 3 }, CancellationToken::new())
        .await
        .unwrap();
    let rejected_n = (0..p.items).filter(|&i| rejected(&p.item(i))).count() as u64;
    assert_eq!(summary.total.rejected, rejected_n);
    assert_eq!(summary.total.done, p.items - rejected_n);
    assert_eq!(summary.total.failed, 0);
    assert_eq!(summary.session, summary.total);
    assert_eq!(summary.remaining, 0);
    assert!(!summary.interrupted);
    assert_eq!(
        summary.reject_reasons,
        vec![("yedinin katı".to_owned(), rejected_n)]
    );
    verify_complete(&dir.0, &p);
    // 25 satırlık shard'lar.
    let shards = fs::read_dir(dir.0.join("shards")).unwrap().count() as u64;
    assert_eq!(shards, (p.items - rejected_n).div_ceil(25));

    let again = Runner::new(p.clone(), &dir.0)
        .provenance(json!({"model": "sahte"}))
        .run(Dummy { max_sleep_ms: 0 }, CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(again.run_id, summary.run_id);
    assert_eq!(again.session.done + again.session.rejected, 0);
    assert_eq!(again.total, summary.total);

    let run: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.0.join("run.json")).unwrap()).unwrap();
    assert_eq!(run["run_id"], summary.run_id.as_str());
    assert_eq!(run["plan_hash"], p.hash());
    assert_eq!(run["plan"]["items"], 120);
    assert_eq!(run["provenance"]["model"], "sahte");
    assert_eq!(run["sessions"].as_array().unwrap().len(), 2);
    assert!(run["sessions"][0]["harness_version"].is_string());
}

#[tokio::test]
async fn torn_line_and_lost_line_are_repaired() {
    let dir = TempDir::new("repair");
    let p = plan(60, 1000, 4);
    Runner::new(p.clone(), &dir.0)
        .run(Dummy { max_sleep_ms: 0 }, CancellationToken::new())
        .await
        .unwrap();
    let shard = dir.0.join("shards").join("shard-00000.jsonl");
    // Son satırı kaybet (shard yazılmadan önce çöktü gibi), sonra yarım satır ekle.
    let text = fs::read_to_string(&shard).unwrap();
    let mut lines: Vec<&str> = text.lines().collect();
    let lost: serde_json::Value = serde_json::from_str(lines.pop().unwrap()).unwrap();
    let mut kept = lines.join("\n");
    kept.push('\n');
    fs::write(&shard, kept).unwrap();
    let mut f = fs::OpenOptions::new().append(true).open(&shard).unwrap();
    f.write_all(br#"{"item":"deneme-000099","index":99,"se"#)
        .unwrap();
    drop(f);

    let summary = Runner::new(p.clone(), &dir.0)
        .run(Dummy { max_sleep_ms: 0 }, CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(summary.repaired, 1, "kaybolan satırın kaydı silinmeli");
    assert_eq!(summary.session.done, 1, "kaybolan örnek yeniden üretilmeli");
    verify_complete(&dir.0, &p);
    let lines = shard_lines(&dir.0);
    assert_eq!(lines[lost["item"].as_str().unwrap()], lost);
}

#[tokio::test]
async fn shard_without_db_record_is_adopted() {
    let dir = TempDir::new("adopt");
    let p = plan(30, 1000, 2);
    Runner::new(p.clone(), &dir.0)
        .run(Dummy { max_sleep_ms: 0 }, CancellationToken::new())
        .await
        .unwrap();
    // Satır yazıldı ama işaretlenmeden çöktü.
    let conn = rusqlite::Connection::open(dir.0.join("state.sqlite")).unwrap();
    let n = conn
        .execute("DELETE FROM items WHERE status = 'done' AND idx < 10", [])
        .unwrap();
    assert!(n > 0);
    drop(conn);
    let summary = Runner::new(p.clone(), &dir.0)
        .run(Dummy { max_sleep_ms: 0 }, CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(summary.repaired, n as u64);
    assert_eq!(
        summary.session.done, 0,
        "shard'daki örnek yeniden çalıştırılmamalı"
    );
    verify_complete(&dir.0, &p);
}

#[tokio::test]
async fn plan_and_provenance_changes_are_refused() {
    let dir = TempDir::new("identity");
    let p = plan(10, 1000, 2);
    Runner::new(p.clone(), &dir.0)
        .provenance(json!({"model": "a", "weights": "sha256:1"}))
        .run(Dummy { max_sleep_ms: 0 }, CancellationToken::new())
        .await
        .unwrap();
    let other = plan(11, 1000, 2);
    let err = Runner::new(other, &dir.0)
        .provenance(json!({"model": "a", "weights": "sha256:1"}))
        .run(Dummy { max_sleep_ms: 0 }, CancellationToken::new())
        .await
        .unwrap_err();
    assert!(matches!(err, HarnessError::PlanChanged { .. }), "{err}");
    let err = Runner::new(p.clone(), &dir.0)
        .provenance(json!({"model": "b", "weights": "sha256:1"}))
        .run(Dummy { max_sleep_ms: 0 }, CancellationToken::new())
        .await
        .unwrap_err();
    assert!(matches!(err, HarnessError::ProvenanceChanged(_)), "{err}");
    // Anahtar sırası önemli değil.
    Runner::new(p, &dir.0)
        .provenance(json!({"weights": "sha256:1", "model": "a"}))
        .run(Dummy { max_sleep_ms: 0 }, CancellationToken::new())
        .await
        .unwrap();
}

struct Flaky {
    calls: AtomicU64,
}

impl Stage for Flaky {
    type Output = serde_json::Value;

    async fn run(&self, item: &Item, _attempt: u32) -> Result<Self::Output, StageError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        if item.index.is_multiple_of(3) {
            Err(StageError::retryable("hep geçici"))
        } else {
            Ok(json!({"ok": true}))
        }
    }
}

#[tokio::test]
async fn exhausted_items_fail_and_are_retried_next_run() {
    let dir = TempDir::new("failed");
    let p = plan(9, 1000, 3);
    let stage = Flaky {
        calls: AtomicU64::new(0),
    };
    let summary = Runner::new(p.clone(), &dir.0)
        .run(stage, CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(summary.total.failed, 3);
    assert_eq!(summary.total.done, 6);
    assert_eq!(summary.remaining, 3);
    let stage = Flaky {
        calls: AtomicU64::new(0),
    };
    let summary = Runner::new(p, &dir.0)
        .run(stage, CancellationToken::new())
        .await
        .unwrap();
    // Yalnızca başarısız üç örnek, ikişer deneme.
    assert_eq!(summary.session.failed, 3);
    assert_eq!(summary.session.done, 0);
    let conn = rusqlite::Connection::open(dir.0.join("state.sqlite")).unwrap();
    let attempts: i64 = conn
        .query_row(
            "SELECT attempts FROM items WHERE id = 'deneme-000000'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(attempts, 4);
}

struct Fatal;

impl Stage for Fatal {
    type Output = serde_json::Value;

    async fn run(&self, item: &Item, _attempt: u32) -> Result<Self::Output, StageError> {
        if item.index == 5 {
            Err(StageError::fatal("model yüklenemedi"))
        } else {
            Ok(json!({"ok": item.index}))
        }
    }
}

#[tokio::test]
async fn fatal_error_stops_the_run() {
    let dir = TempDir::new("fatal");
    let p = plan(1000, 1000, 1);
    let err = Runner::new(p.clone(), &dir.0)
        .run(Fatal, CancellationToken::new())
        .await
        .unwrap_err();
    match err {
        HarnessError::Fatal { item, message } => {
            assert_eq!(item, "deneme-000005");
            assert_eq!(message, "model yüklenemedi");
        }
        other => panic!("{other}"),
    }
    let lines = shard_lines(&dir.0);
    assert_eq!(
        lines.len(),
        5,
        "eşzamanlılık 1: önceki beş örnek yazılmış olmalı"
    );
}

struct NotObject;

impl Stage for NotObject {
    type Output = Vec<u8>;

    async fn run(&self, _item: &Item, _attempt: u32) -> Result<Self::Output, StageError> {
        Ok(vec![1, 2])
    }
}

#[tokio::test]
async fn output_must_be_an_object() {
    let dir = TempDir::new("notobject");
    let err = Runner::new(plan(3, 1000, 1), &dir.0)
        .run(NotObject, CancellationToken::new())
        .await
        .unwrap_err();
    assert!(matches!(err, HarnessError::Fatal { .. }), "{err}");
}

/// Belli sayıda örnek başlattıktan sonra durdurmayı tetikler.
struct StopsAfter {
    token: CancellationToken,
    after: u64,
    started: AtomicU64,
}

impl Stage for StopsAfter {
    type Output = serde_json::Value;

    async fn run(&self, item: &Item, _attempt: u32) -> Result<Self::Output, StageError> {
        if self.started.fetch_add(1, Ordering::SeqCst) + 1 == self.after {
            self.token.cancel();
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        Ok(json!({"text": format!("Örnek {} için sentetik metin.", item.index)}))
    }
}

#[tokio::test]
async fn cancel_finishes_in_flight_items() {
    let dir = TempDir::new("cancel");
    let p = plan(100, 1000, 4);
    let token = CancellationToken::new();
    let stage = StopsAfter {
        token: token.clone(),
        after: 30,
        started: AtomicU64::new(0),
    };
    let summary = Runner::new(p.clone(), &dir.0)
        .run(stage, token)
        .await
        .unwrap();
    assert!(summary.interrupted);
    // Başlatılan her örnek bitirilip yazıldı; yenisi başlatılmadı.
    let lines = shard_lines(&dir.0);
    assert_eq!(lines.len() as u64, summary.total.done);
    assert!(
        (30..=33).contains(&summary.total.done),
        "{}",
        summary.total.done
    );
    assert_eq!(summary.remaining, 100 - summary.total.done);
}
