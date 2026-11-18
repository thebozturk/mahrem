//! Sentetik veri üretimi.
//!
//! Yerel bir LLM metni yer tutucularla yazar (`{KİŞİ_1}`, `{TCKN_1}`);
//! değerleri çekirdeğin üreticileri yerleştirir. Değeri biz koyduğumuz için
//! span'lerin yeri ve etiketi kesindir.
//!
//! Akış: [`spec`] belgeyi planlar → [`prompt`] isteği kurar → LLM yazar →
//! [`template`] yer tutucuları ayrıştırır, [`check`] yer tutucu dışında kalmış
//! veriyi arar → [`fill`] ekle uyumlu değerleri yerleştirir → [`hard`] zor
//! durum dönüşümünü uygular → kayıt altın set doğrulayıcısından geçer.
//! [`assemble`] çalıştırmaları birleştirir, tekilleştirir ve böler.
//!
//! Çalıştırma kılavuzu: `docs/datagen.md`.

pub mod assemble;
pub mod blind;
pub mod check;
pub mod dedup;
pub mod doc;
pub mod error;
pub mod fill;
pub mod handwritten;
pub mod hard;
pub mod harmony;
pub mod job;
pub mod prompt;
pub mod spec;
pub mod stage;
pub mod template;

pub use error::{Error, Result};

/// Üretilen kayıtların yönerge sürümü.
pub const GUIDELINE_VERSION: &str = "0.2.0";
