//! Altın set kaydı.

use mahrem_core::Label;
use serde::{Deserialize, Serialize};

/// Bu crate'in okuduğu ve yazdığı şema sürümü.
pub const SCHEMA_VERSION: u32 = 1;

/// Altın setteki bir belge. JSONL dosyasında bir satır.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoldRecord {
    /// Tüm altın set dosyalarında benzersiz kimlik: `[A-Za-z0-9._-]`, en çok 128 karakter.
    pub id: String,
    /// NFC biçiminde belge metni.
    pub text: String,
    /// Kişisel veri aralıkları; `start`'a göre sıralı, çakışmasız. Kişisel
    /// veri içermeyen belgede boştur.
    pub spans: Vec<GoldSpan>,
    /// Belge hakkında bilgi.
    pub meta: Meta,
}

/// Altın setteki bir kişisel veri aralığı.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoldSpan {
    /// Başlangıç, code point cinsinden.
    pub start: usize,
    /// Bitiş, code point cinsinden, hariç.
    pub end: usize,
    /// Etiket kimliği.
    #[serde(with = "crate::label_serde")]
    pub label: Label,
    /// `text[start..end]`; okuyanın kontrol edebilmesi için tekrar yazılır.
    pub text: String,
    /// Etiketleyici emin değil; şüphede etiketlenir ve işaretlenir.
    pub uncertain: bool,
    /// Değer bir çocuğa ait. Yalnızca `PERSON` ve `AGE` için.
    pub child: bool,
}

/// Belge hakkında bilgi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Meta {
    /// Kaydın şema sürümü; [`SCHEMA_VERSION`] olmalı.
    pub schema_version: u32,
    /// Etiketlerin uyduğu etiketleme yönergesinin sürümü (`0.1.0`).
    pub guideline_version: String,
    /// Metnin nasıl yazıldığı.
    pub source: Source,
    /// Metnin türü.
    pub domain: Domain,
    /// Belgenin içerdiği zor durumlar.
    pub hard_cases: Vec<HardCase>,
    /// Köken: metni hangi üretici ve hangi çalıştırma üretti.
    pub origin: Origin,
}

/// Metnin nasıl yazıldığı.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// Elle yazıldı.
    Handwritten,
    /// Şablon ve değer üreticileriyle üretildi.
    Template,
    /// Bir LLM yer tutuculu metin yazdı, değerleri üreticiler doldurdu.
    Llm,
}

/// Metnin türü.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    /// Müşteri hizmetleri mesajı.
    CustomerSupport,
    /// E-posta.
    Email,
    /// Sohbet ya da anlık mesaj.
    Chat,
    /// Form ya da başvuru.
    Form,
    /// Uzun belge: sözleşme, rapor, dilekçe.
    Document,
    /// Sosyal medya.
    SocialMedia,
    /// Uygulama kaydı (log) ya da kod.
    Log,
    /// Diğer.
    Other,
}

/// Kural katmanını ya da modeli zorlayan durum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HardCase {
    /// Türkçe karaktersiz yazım ("Ayse Yilmaz").
    NoDiacritics,
    /// Tamamı büyük harf.
    AllCaps,
    /// Tamamı küçük harf.
    AllLowercase,
    /// Yazım hatası.
    Typo,
    /// Hem ad hem sözcük olan sözcük (Umut, Deniz, Bahar).
    AmbiguousName,
    /// Kişi adı taşıyan yer ya da kurum ("Atatürk Caddesi").
    NameInPlace,
    /// Yazıyla yazılmış numara ("sıfır beş yüz otuz iki").
    SpelledNumbers,
    /// Araya boşluk ya da noktalama konmuş numara ("0 5 3 2 - 1 2 3").
    SpacedNumbers,
    /// Satırlara bölünmüş numara.
    SplitLines,
    /// "at" ve "nokta" ile yazılmış e-posta.
    ObfuscatedEmail,
}

/// Köken kaydı.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Origin {
    /// Üretici: model adı ya da şablon kimliği. Elle yazılmışsa `null`.
    pub generator: Option<String>,
    /// Üretimi yapan harness çalıştırmasının kimliği.
    pub run_id: Option<String>,
}
