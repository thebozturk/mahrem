use std::path::PathBuf;

use crate::PlanError;

/// Harness hatası.
#[derive(Debug, thiserror::Error)]
pub enum HarnessError {
    /// Plan geçersiz.
    #[error(transparent)]
    Plan(#[from] PlanError),
    /// Dosya hatası.
    #[error("{path}: {source}")]
    Io {
        /// Dosya.
        path: PathBuf,
        /// Hata.
        source: std::io::Error,
    },
    /// SQLite hatası.
    #[error("durum veritabanı: {0}")]
    Store(#[from] rusqlite::Error),
    /// Shard bozuk: ortada okunamayan satır ya da çift kimlik.
    #[error("bozuk shard {path}: {reason}")]
    Corrupt {
        /// Shard.
        path: PathBuf,
        /// Neden.
        reason: String,
    },
    /// Çıktı dizini başka bir planla başlatılmış.
    #[error("çıktı dizini başka bir planla başlatılmış (kayıtlı özet {stored}, bu plan {current})")]
    PlanChanged {
        /// Kayıtlı plan özeti.
        stored: String,
        /// Bu planın özeti.
        current: String,
    },
    /// Kökene eklenen alanlar (model, ağırlık özeti …) önceki oturumdan farklı.
    #[error("köken değişti; yarım kalan bir çalıştırma başka bir modelle sürdürülemez: {0}")]
    ProvenanceChanged(String),
    /// Aşama ölümcül bir hata verdi; çalıştırma durdu.
    #[error("aşama ölümcül hata verdi ({item}): {message}")]
    Fatal {
        /// Hatanın çıktığı örnek.
        item: String,
        /// Hata.
        message: String,
    },
    /// Bir görev panikledi ya da yazıcı durdu.
    #[error("iç hata: {0}")]
    Internal(String),
}

pub(crate) trait IoContext<T> {
    fn at(self, path: impl Into<PathBuf>) -> Result<T, HarnessError>;
}

impl<T> IoContext<T> for std::io::Result<T> {
    fn at(self, path: impl Into<PathBuf>) -> Result<T, HarnessError> {
        self.map_err(|source| HarnessError::Io {
            path: path.into(),
            source,
        })
    }
}
