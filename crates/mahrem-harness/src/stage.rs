//! Aşama: bir örneği işleyen iş.

use std::future::Future;

use serde::Serialize;

use crate::Item;

/// Kutulanmış hata.
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Aşama hatası; harness'ın ne yapacağını belirler.
#[derive(Debug, thiserror::Error)]
pub enum StageError {
    /// Geçici; örnek `max_attempts`'a kadar yeniden denenir. Denemeler
    /// tükenirse örnek başarısız sayılır ve sonraki çalıştırmada yeniden denenir.
    #[error("geçici hata: {0}")]
    Retryable(BoxError),
    /// Örnek reddedildi (örneğin üretilen metin kontrolden geçmedi). Gerekçe
    /// kaydedilir ve gruplanarak raporlanır; kısa ve sabit tutulmalı
    /// (`"json-gecersiz"`, `"span-kaydi"` …). Örnek yeniden çalıştırılmaz.
    #[error("reddedildi: {reason}")]
    Reject {
        /// Gerekçe.
        reason: String,
    },
    /// Ölümcül; yeni örnek başlatılmaz, süren örnekler bitirilir ve
    /// çalıştırma hatayla döner.
    #[error("ölümcül hata: {0}")]
    Fatal(BoxError),
}

impl StageError {
    /// Geçici hata.
    pub fn retryable(e: impl Into<BoxError>) -> Self {
        StageError::Retryable(e.into())
    }

    /// Ret.
    pub fn reject(reason: impl Into<String>) -> Self {
        StageError::Reject {
            reason: reason.into(),
        }
    }

    /// Ölümcül hata.
    pub fn fatal(e: impl Into<BoxError>) -> Self {
        StageError::Fatal(e.into())
    }
}

/// Bir örneği işleyen aşama.
///
/// Aynı örnek ve deneme için aynı çıktıyı vermesi beklenir: tohumu
/// [`Item::seed`]'den ve `attempt`'ten türetin.
pub trait Stage: Send + Sync + 'static {
    /// Shard'a yazılan çıktı; bir JSON nesnesine dönüşmeli.
    type Output: Serialize + Send + 'static;

    /// Örneği işler. `attempt` 0'dan başlar.
    fn run(
        &self,
        item: &Item,
        attempt: u32,
    ) -> impl Future<Output = Result<Self::Output, StageError>> + Send;

    /// İlerleme loglarına eklenecek sayaçlar (örneğin harcanan token).
    fn counters(&self) -> Vec<(&'static str, u64)> {
        Vec::new()
    }
}
