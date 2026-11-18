//! Hata tipi.

use std::path::PathBuf;

/// Veri üretimi hatası.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Dosya hatası.
    #[error("{path}: {source}")]
    Io {
        /// Dosya.
        path: PathBuf,
        /// Hata.
        source: std::io::Error,
    },
    /// Girdi okunamadı ya da geçersiz.
    #[error("{0}")]
    Invalid(String),
    /// Harness hatası.
    #[error(transparent)]
    Harness(#[from] mahrem_harness::HarnessError),
    /// Plan hatası.
    #[error(transparent)]
    Plan(#[from] mahrem_harness::PlanError),
    /// LLM istemcisi kurulamadı.
    #[error(transparent)]
    Llm(#[from] mahrem_llm::LlmError),
}

/// Sonuç tipi.
pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn io(path: impl Into<PathBuf>) -> impl FnOnce(std::io::Error) -> Error {
    let path = path.into();
    move |source| Error::Io { path, source }
}
