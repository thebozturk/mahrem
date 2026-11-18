//! OpenAI uyumlu LLM istemcisi; gateway ve veri üretimi ortak kullanır.
//!
//! - `chat/completions` uç noktasına akışsız istek atar.
//! - Zaman aşımı, 429 ve 5xx'te üstel bekleme ve jitter ile yeniden dener;
//!   429'da `Retry-After`'a uyar.
//! - Eşzamanlı istek sayısını sınırlar ve harcanan token'ları sayar.
//! - Varsayılan olarak yalnızca yerel uçlara bağlanır. Maskelenmemiş veri
//!   yalnızca yerel ya da kurum içi bir modele gidebilir; uzak bir uç açık bir
//!   ayar ister. Gerekçe: `docs/decisions/0009-llm-yerel-uc.md`.
//!
//! ```no_run
//! use mahrem_llm::{ChatRequest, Client, ClientConfig, Message};
//!
//! # async fn demo() -> Result<(), mahrem_llm::LlmError> {
//! let client = Client::new(ClientConfig::local("http://127.0.0.1:8080/v1"))?;
//! let req = ChatRequest::new("gemma", vec![Message::user("Merhaba")]).seed(7);
//! let resp = client.chat(&req).await?;
//! println!("{}", resp.content);
//! # Ok(())
//! # }
//! ```

mod backoff;
mod client;
mod endpoint;
mod error;
mod types;

pub use client::{Client, ClientConfig};
pub use error::LlmError;
pub use types::{ChatRequest, ChatResponse, Message, ResponseFormat, Role, Usage};
