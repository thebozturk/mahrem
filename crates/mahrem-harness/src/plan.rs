//! İş planı.

use serde::Deserialize;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};

/// Plan hatası.
#[derive(Debug, thiserror::Error)]
pub enum PlanError {
    /// TOML okunamadı.
    #[error("plan okunamadı: {0}")]
    Parse(String),
    /// Bir alan geçersiz.
    #[error("plan geçersiz: {0}")]
    Invalid(String),
    /// `[job]` tablosu işin beklediği biçimde değil.
    #[error("[job] okunamadı: {0}")]
    Job(String),
}

/// İş planı. Örnekler ve tohumları plandan deterministik türer.
///
/// ```toml
/// name = "gold-v0"
/// seed = "mahrem-gold-v0"
/// items = 1200
/// shard_size = 1000   # varsayılan 1000
/// concurrency = 4     # varsayılan 4
/// max_attempts = 3    # varsayılan 3
///
/// [job]               # işe özgü ayarlar; üst binary okur
/// model = "gemma"
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    /// Çalıştırma adı; küçük harf, rakam ve `-`. Örnek kimliklerinin önekidir.
    pub name: String,
    /// Tohum.
    pub seed: String,
    /// Örnek sayısı.
    pub items: u64,
    /// Shard başına satır.
    #[serde(default = "default_shard_size")]
    pub shard_size: u64,
    /// Aynı anda işlenen en çok örnek.
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,
    /// Geçici hatada bir örnek için en çok deneme.
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,
    /// İşe özgü ayarlar.
    #[serde(default)]
    pub job: toml::Table,
    #[serde(skip)]
    hash: String,
    #[serde(skip)]
    canonical: serde_json::Value,
}

fn default_shard_size() -> u64 {
    1000
}

fn default_concurrency() -> usize {
    4
}

fn default_max_attempts() -> u32 {
    3
}

/// Planın bir örneği.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// Sıra, `0..items`.
    pub index: u64,
    /// Kimlik: `<ad>-<sıra>`, sıra en az altı hane.
    pub id: String,
    /// Örneğe özgü tohum.
    pub seed: u64,
}

impl Plan {
    /// TOML metninden okur ve denetler.
    ///
    /// # Errors
    ///
    /// TOML geçersizse, bilinmeyen bir alan varsa ya da bir değer kurala
    /// uymuyorsa.
    pub fn from_toml(text: &str) -> Result<Self, PlanError> {
        let value: toml::Value =
            toml::from_str(text).map_err(|e| PlanError::Parse(e.to_string()))?;
        let mut plan: Plan = value
            .clone()
            .try_into()
            .map_err(|e: toml::de::Error| PlanError::Parse(e.to_string()))?;
        plan.validate()?;
        // Özet, biçimden bağımsız olsun diye anahtarları sıralı JSON üzerinden alınır.
        let mut canonical =
            serde_json::to_value(&value).map_err(|e| PlanError::Parse(e.to_string()))?;
        if let serde_json::Value::Object(map) = &mut canonical {
            map.insert("shard_size".into(), plan.shard_size.into());
            map.insert("concurrency".into(), plan.concurrency.into());
            map.insert("max_attempts".into(), plan.max_attempts.into());
            map.entry("job").or_insert_with(|| serde_json::json!({}));
        }
        plan.hash = hex(&Sha256::digest(canonical.to_string().as_bytes()));
        plan.canonical = canonical;
        Ok(plan)
    }

    fn validate(&self) -> Result<(), PlanError> {
        let name_ok = self
            .name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            && self.name.bytes().next().is_some_and(|b| b != b'-');
        if !name_ok {
            return Err(PlanError::Invalid(format!(
                "name {:?}: küçük harf, rakam ve '-' olmalı",
                self.name
            )));
        }
        if self.seed.is_empty() {
            return Err(PlanError::Invalid("seed boş olamaz".into()));
        }
        for (field, value) in [
            ("items", self.items),
            ("shard_size", self.shard_size),
            ("concurrency", self.concurrency as u64),
            ("max_attempts", u64::from(self.max_attempts)),
        ] {
            if value == 0 {
                return Err(PlanError::Invalid(format!("{field} en az 1 olmalı")));
            }
        }
        Ok(())
    }

    /// Planın SHA-256 özeti, küçük harfli onaltılık. Boşluk, yorum ve anahtar
    /// sırası özeti değiştirmez; varsayılan değeri açıkça yazmak da.
    #[must_use]
    pub fn hash(&self) -> &str {
        &self.hash
    }

    /// Planın kanonik JSON hali (köken kaydına yazılır).
    #[must_use]
    pub fn canonical(&self) -> &serde_json::Value {
        &self.canonical
    }

    /// `index`. örnek. Tohum: `SHA-256(seed + "\n" + index)` özetinin ilk 8
    /// baytı, büyük endian.
    #[must_use]
    pub fn item(&self, index: u64) -> Item {
        let mut hasher = Sha256::new();
        hasher.update(self.seed.as_bytes());
        hasher.update(b"\n");
        hasher.update(index.to_string().as_bytes());
        let hash = hasher.finalize();
        let mut first = [0u8; 8];
        first.copy_from_slice(&hash[..8]);
        Item {
            index,
            id: format!("{}-{index:06}", self.name),
            seed: u64::from_be_bytes(first),
        }
    }

    /// `[job]` tablosunu işin ayar tipine çevirir.
    ///
    /// # Errors
    ///
    /// Tablo `T`'ye uymuyorsa.
    pub fn job<T: DeserializeOwned>(&self) -> Result<T, PlanError> {
        toml::Value::Table(self.job.clone())
            .try_into()
            .map_err(|e: toml::de::Error| PlanError::Job(e.to_string()))
    }
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut out, b| {
            let _ = write!(out, "{b:02x}");
            out
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAN: &str = r#"
name = "deneme"
seed = "tohum"
items = 10

[job]
model = "gemma"
temperature = 0.7
"#;

    #[test]
    fn parses_with_defaults() {
        let p = Plan::from_toml(PLAN).unwrap();
        assert_eq!(p.items, 10);
        assert_eq!(p.shard_size, 1000);
        assert_eq!(p.concurrency, 4);
        assert_eq!(p.max_attempts, 3);
        assert_eq!(p.hash().len(), 64);
    }

    #[test]
    fn hash_ignores_formatting_and_defaults() {
        let a = Plan::from_toml(PLAN).unwrap();
        let b = Plan::from_toml("# yorum\n[job]\ntemperature = 0.7\nmodel = \"gemma\"\n\n[meta]\n");
        assert!(b.is_err(), "bilinmeyen tablo reddedilmeli");
        let b = Plan::from_toml(
            "items = 10\nseed = \"tohum\"\nshard_size = 1000\nname = \"deneme\"\n[job]\ntemperature = 0.7\nmodel = \"gemma\"\n",
        )
        .unwrap();
        assert_eq!(a.hash(), b.hash());
        let c = Plan::from_toml(&PLAN.replace("0.7", "0.8")).unwrap();
        assert_ne!(a.hash(), c.hash());
        let d = Plan::from_toml(&PLAN.replace("items = 10", "items = 11")).unwrap();
        assert_ne!(a.hash(), d.hash());
    }

    #[test]
    fn items_are_deterministic() {
        let p = Plan::from_toml(PLAN).unwrap();
        let a = p.item(3);
        assert_eq!(a.id, "deneme-000003");
        assert_eq!(a, p.item(3));
        assert_ne!(a.seed, p.item(4).seed);
        let other = Plan::from_toml(&PLAN.replace("tohum", "başka")).unwrap();
        assert_ne!(a.seed, other.item(3).seed);
        assert_eq!(p.item(1_234_567).id, "deneme-1234567");
    }

    #[test]
    fn job_table_is_typed() {
        #[derive(Deserialize)]
        struct Job {
            model: String,
            temperature: f64,
        }
        #[derive(Deserialize, Debug)]
        #[allow(dead_code)]
        struct Other {
            prompt: String,
        }
        let p = Plan::from_toml(PLAN).unwrap();
        let job: Job = p.job().unwrap();
        assert_eq!(job.model, "gemma");
        assert!((job.temperature - 0.7).abs() < 1e-12);
        assert!(matches!(p.job::<Other>(), Err(PlanError::Job(_))));
    }

    #[test]
    fn invalid_plans() {
        for (from, to) in [
            ("\"deneme\"", "\"Deneme\""),
            ("\"deneme\"", "\"-deneme\""),
            ("\"deneme\"", "\"\""),
            ("\"tohum\"", "\"\""),
            ("items = 10", "items = 0"),
            ("items = 10", "items = -1"),
            ("items = 10", "items = 10\nconcurrency = 0"),
            ("items = 10", "items = 10\nbilinmeyen = 1"),
        ] {
            assert!(Plan::from_toml(&PLAN.replace(from, to)).is_err(), "{to}");
        }
    }
}
