//! Kaldığı yerden devam: harness ayrı bir süreçte çalıştırılır ve rastgele
//! anlarda SIGKILL ile öldürülür. Sonunda kayıp ya da çift örnek olmamalı.
//!
//! Alt süreç bu test dosyasının kendisidir: `crash_child` testi yalnızca
//! ortam değişkeni verildiğinde çalışır.

mod common;

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use common::{Dummy, TempDir, plan, verify_complete};
use mahrem_harness::{CancellationToken, Runner};

const DIR_ENV: &str = "MAHREM_HARNESS_CRASH_DIR";
const ITEMS: u64 = 600;

fn child_plan() -> mahrem_harness::Plan {
    plan(ITEMS, 37, 8)
}

#[test]
#[ignore = "crash testinin alt süreci; doğrudan çalıştırılmaz"]
fn crash_child() {
    let Ok(dir) = std::env::var(DIR_ENV) else {
        return;
    };
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        Runner::new(child_plan(), PathBuf::from(dir))
            .run(Dummy { max_sleep_ms: 40 }, CancellationToken::new())
            .await
            .unwrap();
    });
}

fn spawn(dir: &std::path::Path) -> std::process::Child {
    Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "crash_child", "--ignored", "--quiet"])
        .env(DIR_ENV, dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
}

fn splitmix64(x: &mut u64) -> u64 {
    *x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *x;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

#[test]
fn survives_random_kills() {
    let dir = TempDir::new("crash");
    let mut rng = u64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            % u128::from(u64::MAX),
    )
    .unwrap();
    eprintln!("crash testi tohumu: {rng}");
    let mut kills = 0;
    let mut finished = false;
    for _ in 0..100 {
        let mut child = spawn(&dir.0);
        let wait = Duration::from_millis(20 + splitmix64(&mut rng) % 100);
        let start = Instant::now();
        while start.elapsed() < wait {
            if child.try_wait().unwrap().is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "alt süreç hatayla bitti: {status}");
            finished = true;
            break;
        }
        child.kill().unwrap();
        child.wait().unwrap();
        kills += 1;
    }
    if !finished {
        let status = spawn(&dir.0).wait().unwrap();
        assert!(status.success(), "son çalıştırma hatayla bitti: {status}");
    }
    assert!(kills >= 8, "süreç yeterince öldürülmedi ({kills})");
    eprintln!("{kills} kez öldürüldü");
    verify_complete(&dir.0, &child_plan());
}
