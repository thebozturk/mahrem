//! Köken kaydı: `run.json`.

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::IoContext as _;
use crate::plan::hex;
use crate::{HarnessError, Plan};

/// Harness'ın derlendiği commit, biliniyorsa.
pub(crate) const GIT_SHA: Option<&str> = option_env!("MAHREM_GIT_SHA");

#[derive(Serialize, Deserialize)]
struct RunFile {
    run_id: String,
    plan_name: String,
    plan_hash: String,
    plan: serde_json::Value,
    /// Üst binary'nin eklediği alanlar: model, ağırlık özeti, örnekleme
    /// parametreleri, şablon özeti, uzak uç izni …
    provenance: serde_json::Value,
    sessions: Vec<Session>,
}

#[derive(Serialize, Deserialize)]
struct Session {
    /// Başlangıç, Unix saniyesi.
    started_at: u64,
    harness_version: String,
    harness_git_sha: Option<String>,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Yeni bir çalıştırma kimliği: `<ad>-<12 hane>`.
pub(crate) fn new_run_id(plan: &Plan) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let mut h = Sha256::new();
    h.update(plan.hash().as_bytes());
    h.update(nanos.to_le_bytes());
    h.update(std::process::id().to_le_bytes());
    format!("{}-{}", plan.name, &hex(&h.finalize())[..12])
}

/// `run.json`'u yazar ya da yeni oturumu ekler. Dosya geçici bir adla yazılıp
/// yerine taşınır; yarım kalmaz.
pub(crate) fn record_session(
    out_dir: &Path,
    run_id: &str,
    plan: &Plan,
    extra: &serde_json::Value,
) -> Result<(), HarnessError> {
    let path = out_dir.join("run.json");
    let mut file = match fs::read(&path) {
        Ok(bytes) => {
            serde_json::from_slice::<RunFile>(&bytes).map_err(|e| HarnessError::Corrupt {
                path: path.clone(),
                reason: e.to_string(),
            })?
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => RunFile {
            run_id: run_id.to_owned(),
            plan_name: plan.name.clone(),
            plan_hash: plan.hash().to_owned(),
            plan: plan.canonical().clone(),
            provenance: extra.clone(),
            sessions: Vec::new(),
        },
        Err(e) => return Err(e).at(&path),
    };
    file.sessions.push(Session {
        started_at: now(),
        harness_version: env!("CARGO_PKG_VERSION").to_owned(),
        harness_git_sha: GIT_SHA.map(str::to_owned),
    });
    let mut text = serde_json::to_string_pretty(&file).expect("run.json yazılamadı");
    text.push('\n');
    let tmp = out_dir.join("run.json.tmp");
    fs::write(&tmp, text).at(&tmp)?;
    fs::rename(&tmp, &path).at(&path)?;
    Ok(())
}
