//! Planın `[job]` tablosu.

use serde::Deserialize;

/// Üretim işinin ayarları.
///
/// ```toml
/// [job]
/// endpoint = "http://127.0.0.1:8080/v1"
/// model = "<sunucunun bildiği model adı>"
/// model_revision = "<Hugging Face commit SHA'sı>"
/// temperature = 0.9
/// max_tokens = 700
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Job {
    /// OpenAI uyumlu uç; varsayılan olarak yalnızca yerel.
    pub endpoint: String,
    /// Model adı.
    pub model: String,
    /// Ağırlıkların sürümü (Hugging Face commit SHA'sı); köken kaydına girer.
    pub model_revision: String,
    /// Yerel olmayan uca izin (karar 0009).
    #[serde(default)]
    pub allow_remote: bool,
    /// Örnekleme sıcaklığı.
    #[serde(default = "default_temperature")]
    pub temperature: f64,
    /// Çekirdek örnekleme eşiği.
    #[serde(default)]
    pub top_p: Option<f64>,
    /// En çok token.
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    /// Tek isteğin süre sınırı, saniye.
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_temperature() -> f64 {
    0.9
}

fn default_max_tokens() -> u32 {
    700
}

fn default_timeout() -> u64 {
    300
}

impl Job {
    /// Köken kaydına yazılacak alanlar.
    #[must_use]
    pub fn provenance(&self) -> serde_json::Value {
        serde_json::json!({
            "datagen_version": env!("CARGO_PKG_VERSION"),
            "model": self.model,
            "model_revision": self.model_revision,
            "allow_remote": self.allow_remote,
            "temperature": self.temperature,
            "top_p": self.top_p,
            "max_tokens": self.max_tokens,
            "template_hash": crate::spec::template_hash(),
            "guideline_version": crate::GUIDELINE_VERSION,
        })
    }
}
