//! Kör okuma dosyası: birkaç çalıştırmanın metinlerini model adını
//! göstermeden karıştırır. Model seçimi için.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::Result;
use crate::assemble::load_run;
use crate::error::io;

fn key(seed: &str, s: &str) -> [u8; 32] {
    Sha256::digest(format!("{seed}\n{s}").as_bytes()).into()
}

/// Her çalıştırmadan `per_run` metin seçer, karıştırır; `metinler.md` ve
/// hangi metnin hangi modelden geldiğini tutan `anahtar.json`'u yazar.
/// Yazılan metin sayısını döner.
///
/// # Errors
///
/// Bir çalıştırma okunamaz ya da dosya yazılamazsa.
pub fn blind(runs: &[PathBuf], per_run: usize, seed: &str, out: &Path) -> Result<usize> {
    let mut picked = Vec::new();
    for dir in runs {
        let (run_id, provenance, mut records) = load_run(dir)?;
        records.sort_by_key(|r| key(seed, &r.id));
        let model = provenance["model"].as_str().unwrap_or("?").to_owned();
        for r in records.into_iter().take(per_run) {
            picked.push((
                key(seed, &format!("{run_id}/{}", r.id)),
                model.clone(),
                run_id.clone(),
                r,
            ));
        }
    }
    picked.sort_by_key(|p| p.0);
    fs::create_dir_all(out).map_err(io(out))?;
    let mut md = String::from(
        "# Kör okuma\n\nHer metni 1-5 arası puanla: Türkçesi ne kadar doğal, gerçek bir insan böyle yazar mı?\n",
    );
    let mut answer = Vec::new();
    for (i, (_, model, run_id, r)) in picked.iter().enumerate() {
        let n = i + 1;
        let _ = write!(md, "\n## {n}\n\n```text\n{}\n```\n\nPuan:\n", r.text);
        answer.push(serde_json::json!({"no": n, "model": model, "run_id": run_id, "id": r.id}));
    }
    let md_path = out.join("metinler.md");
    fs::write(&md_path, md).map_err(io(&md_path))?;
    let key_path = out.join("anahtar.json");
    let text = serde_json::to_string_pretty(&answer).unwrap_or_default();
    fs::write(&key_path, text + "\n").map_err(io(&key_path))?;
    Ok(picked.len())
}
