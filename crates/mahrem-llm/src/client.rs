//! İstemci.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use reqwest::header::{CONTENT_TYPE, RETRY_AFTER};
use reqwest::{StatusCode, Url};
use tokio::sync::Semaphore;
use tracing::warn;

use crate::backoff::{delay, splitmix64};
use crate::endpoint::chat_url;
use crate::error::body_excerpt;
use crate::types::RawResponse;
use crate::{ChatRequest, ChatResponse, LlmError, Usage};

/// İstemci ayarları.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Taban adres, örneğin `http://127.0.0.1:8080/v1`.
    pub base_url: String,
    /// `Authorization: Bearer` anahtarı, gerekiyorsa.
    pub api_key: Option<String>,
    /// Tek bir isteğin süre sınırı.
    pub timeout: Duration,
    /// Geçici hatalarda en çok yeniden deneme sayısı.
    pub max_retries: u32,
    /// İlk yeniden denemeden önceki bekleme.
    pub backoff_base: Duration,
    /// En uzun bekleme. Sunucu `Retry-After` ile daha uzununu isterse
    /// beklenmez, hata döner.
    pub backoff_max: Duration,
    /// Aynı anda en çok istek sayısı.
    pub concurrency: usize,
    /// Yerel olmayan uca izin ver. Maskelenmemiş veriyle yalnızca kurum içi
    /// bir uç için açılır; açıldığı köken kaydına yazılmalıdır.
    pub allow_remote: bool,
    /// Jitter tohumu.
    pub jitter_seed: u64,
}

impl ClientConfig {
    /// Yerel bir uç için varsayılan ayarlar.
    pub fn local(base_url: impl Into<String>) -> Self {
        ClientConfig {
            base_url: base_url.into(),
            api_key: None,
            timeout: Duration::from_secs(120),
            max_retries: 4,
            backoff_base: Duration::from_millis(500),
            backoff_max: Duration::from_secs(30),
            concurrency: 4,
            allow_remote: false,
            jitter_seed: 0,
        }
    }
}

/// OpenAI uyumlu istemci.
#[derive(Debug)]
pub struct Client {
    http: reqwest::Client,
    url: Url,
    config: ClientConfig,
    permits: Semaphore,
    requests: AtomicU64,
    prompt_tokens: AtomicU64,
    completion_tokens: AtomicU64,
}

impl Client {
    /// İstemciyi kurar.
    ///
    /// # Errors
    ///
    /// Adres geçersizse, `https` ise ya da yerel değilken `allow_remote`
    /// kapalıysa; `concurrency` sıfırsa.
    pub fn new(config: ClientConfig) -> Result<Self, LlmError> {
        let url = chat_url(&config.base_url, config.allow_remote)?;
        if config.concurrency == 0 {
            return Err(LlmError::Config("concurrency en az 1 olmalı".into()));
        }
        // Vekil sunucu ve yönlendirme kapalı: istek yalnızca denetlenen adrese gider.
        let http = reqwest::Client::builder()
            .timeout(config.timeout)
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| LlmError::Config(e.to_string()))?;
        Ok(Client {
            http,
            url,
            permits: Semaphore::new(config.concurrency),
            config,
            requests: AtomicU64::new(0),
            prompt_tokens: AtomicU64::new(0),
            completion_tokens: AtomicU64::new(0),
        })
    }

    /// İsteklerin gittiği adres.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        self.url.as_str()
    }

    /// Uzak uca izin verildi mi?
    #[must_use]
    pub fn allows_remote(&self) -> bool {
        self.config.allow_remote
    }

    /// Şimdiye kadar başarılı isteklerde harcanan token'lar.
    #[must_use]
    pub fn usage(&self) -> Usage {
        Usage {
            prompt_tokens: self.prompt_tokens.load(Ordering::Relaxed),
            completion_tokens: self.completion_tokens.load(Ordering::Relaxed),
        }
    }

    /// Bir sohbet isteği gönderir; geçici hatalarda yeniden dener.
    ///
    /// # Errors
    ///
    /// Kalıcı bir hata ([`LlmError::Rejected`], [`LlmError::Malformed`]) ya
    /// da yeniden denemeler tükenince [`LlmError::Exhausted`].
    pub async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse, LlmError> {
        let body = serde_json::to_vec(req).map_err(|e| LlmError::Config(e.to_string()))?;
        let _permit = self
            .permits
            .acquire()
            .await
            .map_err(|e| LlmError::Config(e.to_string()))?;
        let n = self.requests.fetch_add(1, Ordering::Relaxed);
        let mut attempt = 0;
        loop {
            let err = match self.send(&body).await {
                Ok(resp) => {
                    self.prompt_tokens
                        .fetch_add(resp.usage.prompt_tokens, Ordering::Relaxed);
                    self.completion_tokens
                        .fetch_add(resp.usage.completion_tokens, Ordering::Relaxed);
                    return Ok(resp);
                }
                Err(err) if !err.is_retryable() => return Err(err),
                Err(err) => err,
            };
            let wait = match &err {
                LlmError::RateLimited {
                    retry_after: Some(d),
                } => {
                    if *d > self.config.backoff_max {
                        return Err(err);
                    }
                    *d
                }
                _ => delay(
                    self.config.backoff_base,
                    self.config.backoff_max,
                    attempt,
                    splitmix64(self.config.jitter_seed ^ n.rotate_left(32) ^ u64::from(attempt)),
                ),
            };
            if attempt >= self.config.max_retries {
                return Err(LlmError::Exhausted {
                    attempts: attempt + 1,
                    last: Box::new(err),
                });
            }
            warn!(error = %err, attempt, wait_ms = wait.as_millis(), "LLM isteği yeniden denenecek");
            tokio::time::sleep(wait).await;
            attempt += 1;
        }
    }

    async fn send(&self, body: &[u8]) -> Result<ChatResponse, LlmError> {
        let mut rb = self
            .http
            .post(self.url.clone())
            .header(CONTENT_TYPE, "application/json")
            .body(body.to_vec());
        if let Some(key) = &self.config.api_key {
            rb = rb.bearer_auth(key);
        }
        let resp = rb.send().await.map_err(transport)?;
        let status = resp.status();
        if status == StatusCode::TOO_MANY_REQUESTS {
            let retry_after = resp
                .headers()
                .get(RETRY_AFTER)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.trim().parse::<u64>().ok())
                .map(Duration::from_secs);
            return Err(LlmError::RateLimited { retry_after });
        }
        let bytes = resp.bytes().await.map_err(transport)?;
        if status.is_server_error() {
            return Err(LlmError::Server {
                status: status.as_u16(),
                body: body_excerpt(&bytes),
            });
        }
        if !status.is_success() {
            return Err(LlmError::Rejected {
                status: status.as_u16(),
                body: body_excerpt(&bytes),
            });
        }
        let raw: RawResponse = serde_json::from_slice(&bytes)
            .map_err(|e| LlmError::Malformed(format!("{e}: {}", body_excerpt(&bytes))))?;
        raw.into_response().map_err(LlmError::Malformed)
    }
}

#[allow(clippy::needless_pass_by_value)] // `map_err` sahipliği verir.
fn transport(e: reqwest::Error) -> LlmError {
    if e.is_timeout() {
        LlmError::Timeout
    } else {
        LlmError::Transport(e.to_string())
    }
}
