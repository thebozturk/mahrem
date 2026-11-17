//! Mahrem çekirdeği.
//!
//! Türkçe metindeki kişisel verileri tespit eden, maskeleyen ve LLM yanıtında
//! geri yerleştiren motorun Rust çekirdeği. Python kütüphanesi, gateway ve
//! benchmark aynı kodu çalıştırır.
//!
//! Şu an bulunanlar:
//!
//! - [`validators`]: Türkiye'ye özgü kimlik ve finans numaralarının checksum doğrulaması.
//! - [`case`]: Türkçe büyük/küçük harf dönüşümü (`İ` ↔ `i`, `I` ↔ `ı`).

pub mod case;
pub mod validators;

pub use case::{to_lowercase_tr, to_uppercase_tr};
pub use validators::{
    is_valid_card_number, is_valid_iban_tr, is_valid_tckn, is_valid_vkn, is_valid_ykn,
};
