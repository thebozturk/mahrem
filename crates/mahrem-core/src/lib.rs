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
//! - [`label`]: kişisel veri etiketleri, kimlikleri ve placeholder adları.
//! - [`offset`]: code point ile bayt konumu arasında dönüşüm.
//! - [`fake`]: tohumlu sahte değer üreticileri (sentetik veri ve sahte veri operatörü).

pub mod case;
pub mod fake;
pub mod label;
pub mod offset;
pub mod validators;

pub use case::{to_lowercase_tr, to_uppercase_tr};
pub use label::{Label, UnknownLabel};
pub use offset::CodePointIndex;
pub use validators::{
    is_valid_card_number, is_valid_iban_tr, is_valid_tckn, is_valid_vkn, is_valid_ykn,
};
