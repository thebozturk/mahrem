use std::time::Duration;

/// İstemci hatası.
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    /// Yapılandırma geçersiz.
    #[error("yapılandırma hatası: {0}")]
    Config(String),
    /// Uç yerel değil ve uzak uca açıkça izin verilmedi.
    #[error("uç yerel değil: {0}; uzak bir uç için `allow_remote` açıkça verilmeli")]
    RemoteEndpoint(String),
    /// İstek zaman aşımına uğradı.
    #[error("istek zaman aşımına uğradı")]
    Timeout,
    /// Bağlantı kurulamadı ya da koptu.
    #[error("bağlantı hatası: {0}")]
    Transport(String),
    /// Sunucu hız sınırına takıldı (429).
    #[error("hız sınırı (429)")]
    RateLimited {
        /// Sunucunun istediği bekleme (`Retry-After`), varsa.
        retry_after: Option<Duration>,
    },
    /// Sunucu hatası (5xx).
    #[error("sunucu hatası {status}: {body}")]
    Server {
        /// HTTP durum kodu.
        status: u16,
        /// Yanıt gövdesinin başı.
        body: String,
    },
    /// İstek reddedildi (429 dışındaki 4xx); yeniden denenmez.
    #[error("istek reddedildi {status}: {body}")]
    Rejected {
        /// HTTP durum kodu.
        status: u16,
        /// Yanıt gövdesinin başı.
        body: String,
    },
    /// Yanıt beklenen biçimde değil; yeniden denenmez.
    #[error("bozuk yanıt: {0}")]
    Malformed(String),
    /// Yeniden denemeler tükendi.
    #[error("{attempts} denemeden sonra vazgeçildi: {last}")]
    Exhausted {
        /// Toplam deneme sayısı.
        attempts: u32,
        /// Son hata.
        last: Box<LlmError>,
    },
}

impl LlmError {
    /// Hata geçici mi? Geçici hatalar istemcinin içinde yeniden denenir.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            LlmError::Timeout
                | LlmError::Transport(_)
                | LlmError::RateLimited { .. }
                | LlmError::Server { .. }
        )
    }
}

/// Hata mesajına konacak gövdenin başı.
pub(crate) fn body_excerpt(bytes: &[u8]) -> String {
    const MAX: usize = 500;
    let text = String::from_utf8_lossy(bytes);
    match text.char_indices().nth(MAX) {
        Some((i, _)) => format!("{}…", &text[..i]),
        None => text.into_owned(),
    }
}
