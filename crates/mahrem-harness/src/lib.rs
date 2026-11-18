//! Toplu iş çalıştırıcı: deterministik plan, kaldığı yerden devam, köken kaydı.
//!
//! Bir iş, TOML bir [`Plan`] ve her örneği işleyen bir [`Stage`]'den oluşur.
//! [`Runner`] örnekleri sınırlı eşzamanlılıkla çalıştırır:
//!
//! - Kabul edilen çıktılar append-only JSONL shard'larına yazılır, diske
//!   indirilir, sonra SQLite'ta işaretlenir.
//! - Açılışta shard'lar ile SQLite uzlaştırılır: yarım kalmış son satır
//!   kesilir, shard'daki her örnek bitmiş sayılır. Süreç herhangi bir anda
//!   öldürülse de kayıp ya da çift örnek olmaz.
//! - Bitmiş ve reddedilmiş örnekler yeniden çalıştırılmaz.
//! - Her çalıştırmanın kökeni `run.json`'a yazılır.
//!
//! Harness LLM'i bilmez; LLM'i aşamayı yazan üst binary çağırır.
//! Tasarım: `docs/decisions/0008-harness.md`.
//!
//! Çıktı dizini:
//!
//! ```text
//! <out>/run.json            köken
//! <out>/state.sqlite        örnek durumları
//! <out>/shards/shard-00000.jsonl
//! ```
//!
//! Shard satırı: `{"item":"<kimlik>","index":0,"seed":…,"attempt":0,"output":{…}}`.

mod error;
mod logging;
mod plan;
mod provenance;
mod runner;
mod shard;
mod stage;
mod store;

pub use error::HarnessError;
pub use logging::init_tracing;
pub use plan::{Item, Plan, PlanError};
pub use runner::{RunSummary, Runner, cancel_on_ctrl_c};
pub use stage::{BoxError, Stage, StageError};
pub use tokio_util::sync::CancellationToken;
