//! Append-only JSONL shard'ları.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::HarnessError;
use crate::error::IoContext as _;

/// Shard satırı.
#[derive(Serialize)]
pub(crate) struct Line<'a, T> {
    pub(crate) item: &'a str,
    pub(crate) index: u64,
    pub(crate) seed: u64,
    pub(crate) attempt: u32,
    pub(crate) output: &'a T,
}

#[derive(Deserialize)]
struct LineKey {
    item: String,
    index: u64,
}

pub(crate) fn shard_name(n: u64) -> String {
    format!("shard-{n:05}.jsonl")
}

fn shard_number(name: &str) -> Option<u64> {
    name.strip_prefix("shard-")?
        .strip_suffix(".jsonl")?
        .parse()
        .ok()
}

/// Uzlaştırmanın sonucu.
#[derive(Debug, Default)]
pub(crate) struct Scan {
    /// Shard'lardaki örnekler: kimlik → (sıra, shard adı).
    pub(crate) items: HashMap<String, (u64, String)>,
    /// Son shard'ın numarası ve satır sayısı.
    pub(crate) last: Option<(u64, u64)>,
    /// Kesilen yarım satırın bayt sayısı.
    pub(crate) truncated: u64,
}

/// Shard'ları okur. Son shard'ın sonundaki yarım satırı keser (yalnızca
/// orada olabilir: yazım her zaman son shard'a yapılır). Ortada okunamayan
/// bir satır ya da çift kimlik hatadır.
pub(crate) fn scan(dir: &Path) -> Result<Scan, HarnessError> {
    let mut numbers = Vec::new();
    for entry in fs::read_dir(dir).at(dir)? {
        let entry = entry.at(dir)?;
        if let Some(n) = entry.file_name().to_str().and_then(shard_number) {
            numbers.push(n);
        }
    }
    numbers.sort_unstable();
    let mut scan = Scan::default();
    for (pos, &n) in numbers.iter().enumerate() {
        let name = shard_name(n);
        let path = dir.join(&name);
        let mut bytes = fs::read(&path).at(&path)?;
        if bytes.last().is_some_and(|&b| b != b'\n') {
            if pos + 1 != numbers.len() {
                return Err(corrupt(&path, "son olmayan shard yarım satırla bitiyor"));
            }
            let keep = bytes.iter().rposition(|&b| b == b'\n').map_or(0, |i| i + 1);
            scan.truncated = (bytes.len() - keep) as u64;
            let file = OpenOptions::new().write(true).open(&path).at(&path)?;
            file.set_len(keep as u64).at(&path)?;
            file.sync_all().at(&path)?;
            bytes.truncate(keep);
        }
        let mut count = 0;
        for (i, line) in bytes.split(|&b| b == b'\n').enumerate() {
            if line.is_empty() {
                continue;
            }
            let key: LineKey = serde_json::from_slice(line)
                .map_err(|e| corrupt(&path, &format!("{}. satır okunamadı: {e}", i + 1)))?;
            if scan.items.contains_key(&key.item) {
                return Err(corrupt(&path, &format!("{} iki kez yazılmış", key.item)));
            }
            scan.items.insert(key.item, (key.index, name.clone()));
            count += 1;
        }
        scan.last = Some((n, count));
    }
    Ok(scan)
}

fn corrupt(path: &Path, reason: &str) -> HarnessError {
    HarnessError::Corrupt {
        path: path.to_owned(),
        reason: reason.to_owned(),
    }
}

/// Shard yazıcısı. Her satır tek bir yazımla eklenir ve diske indirilir.
pub(crate) struct Writer {
    dir: PathBuf,
    shard_size: u64,
    current: u64,
    lines: u64,
    file: Option<File>,
}

impl Writer {
    pub(crate) fn new(dir: PathBuf, shard_size: u64, last: Option<(u64, u64)>) -> Self {
        let (current, lines) = last.unwrap_or((0, 0));
        Writer {
            dir,
            shard_size,
            current,
            lines,
            file: None,
        }
    }

    /// Satırı ekler (sonuna `\n` koyar), diske indirir ve shard adını döner.
    pub(crate) fn append(&mut self, line: &[u8]) -> Result<String, HarnessError> {
        if self.lines >= self.shard_size {
            self.current += 1;
            self.lines = 0;
            self.file = None;
        }
        let name = shard_name(self.current);
        let path = self.dir.join(&name);
        if self.file.is_none() {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .at(&path)?;
            self.file = Some(file);
        }
        let file = self.file.as_mut().expect("shard açık");
        let mut buf = Vec::with_capacity(line.len() + 1);
        buf.extend_from_slice(line);
        buf.push(b'\n');
        file.write_all(&buf).at(&path)?;
        file.sync_data().at(&path)?;
        self.lines += 1;
        Ok(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "mahrem-shard-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn line(id: &str, index: u64) -> Vec<u8> {
        format!(r#"{{"item":"{id}","index":{index},"seed":1,"attempt":0,"output":{{}}}}"#)
            .into_bytes()
    }

    #[test]
    fn rotates_and_rescans() {
        let dir = tmp("rotate");
        let mut w = Writer::new(dir.clone(), 2, None);
        let names: Vec<String> = (0..5)
            .map(|i| w.append(&line(&format!("x-{i}"), i)).unwrap())
            .collect();
        assert_eq!(names[0], "shard-00000.jsonl");
        assert_eq!(names[2], "shard-00001.jsonl");
        assert_eq!(names[4], "shard-00002.jsonl");
        let s = scan(&dir).unwrap();
        assert_eq!(s.items.len(), 5);
        assert_eq!(s.items["x-3"], (3, "shard-00001.jsonl".into()));
        assert_eq!(s.last, Some((2, 1)));
        // Kaldığı yerden: son shard dolana kadar ona yazar.
        let mut w = Writer::new(dir.clone(), 2, s.last);
        assert_eq!(w.append(&line("x-5", 5)).unwrap(), "shard-00002.jsonl");
        assert_eq!(w.append(&line("x-6", 6)).unwrap(), "shard-00003.jsonl");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn torn_last_line_is_cut() {
        let dir = tmp("torn");
        let mut w = Writer::new(dir.clone(), 10, None);
        w.append(&line("x-0", 0)).unwrap();
        let path = dir.join(shard_name(0));
        let mut f = OpenOptions::new().append(true).open(&path).unwrap();
        f.write_all(br#"{"item":"x-1","ind"#).unwrap();
        drop(f);
        let s = scan(&dir).unwrap();
        assert_eq!(s.items.len(), 1);
        assert_eq!(s.truncated, 18);
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.ends_with('\n'));
        assert_eq!(text.lines().count(), 1);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn corruption_is_reported() {
        let dir = tmp("corrupt");
        fs::write(
            dir.join(shard_name(0)),
            [line("x-0", 0), line("x-0", 0), Vec::new()].join(&b'\n'),
        )
        .unwrap();
        assert!(matches!(scan(&dir), Err(HarnessError::Corrupt { .. })));
        fs::write(dir.join(shard_name(0)), b"bozuk\n").unwrap();
        assert!(matches!(scan(&dir), Err(HarnessError::Corrupt { .. })));
        fs::write(dir.join(shard_name(0)), b"{\"item\":\"x-0\"").unwrap();
        fs::write(dir.join(shard_name(1)), line("x-1", 1)).unwrap();
        assert!(matches!(scan(&dir), Err(HarnessError::Corrupt { .. })));
        fs::remove_dir_all(dir).unwrap();
    }
}
