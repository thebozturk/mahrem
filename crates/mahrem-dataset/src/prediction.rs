//! Benchmark'a verilen araç çıktısı.

use mahrem_core::Label;
use serde::{Deserialize, Serialize};

/// Bir belge için aracın bulduğu aralıklar. Belge metni taşınmaz; kayıt
/// altın setteki belgeye `id` ile bağlanır.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredictionRecord {
    /// Altın setteki belge kimliği.
    pub id: String,
    /// Bulunan aralıklar. Sıra ve çakışma serbesttir.
    pub spans: Vec<PredictedSpan>,
}

/// Aracın bulduğu bir aralık.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredictedSpan {
    /// Başlangıç, code point cinsinden.
    pub start: usize,
    /// Bitiş, code point cinsinden, hariç.
    pub end: usize,
    /// Etiket kimliği.
    #[serde(with = "crate::label_serde")]
    pub label: Label,
    /// İsteğe bağlı; verilirse altın metindeki dilimle eşleşmelidir.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// İsteğe bağlı güven skoru, `[0, 1]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
}
