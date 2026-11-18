//! Yer tutucu dışında kalmış kişisel veri araması.
//!
//! LLM'e yalnızca yer tutucu yazması söylenir, ama kendi kafasından bir
//! numara ya da ad yazabilir. O değer etiketsiz kalır ve altın seti bozar.
//! Burada yakalananlar reddedilir; kalanları insan onayı yakalar.

use mahrem_core::fake::{is_ambiguous_name, is_listed_name};
use mahrem_core::{is_valid_tckn, is_valid_vkn};

use crate::template::Piece;

/// Düz metin parçalarında kişisel veri izi arar; bulursa ret gerekçesi döner.
#[must_use]
pub fn leak(pieces: &[Piece]) -> Option<&'static str> {
    for piece in pieces {
        let Piece::Text(t) = piece else { continue };
        if has_email(t) {
            return Some("ham-eposta");
        }
        if has_number(t) {
            return Some("ham-numara");
        }
        if has_name(t) {
            return Some("ham-ad");
        }
    }
    None
}

fn has_email(t: &str) -> bool {
    let chars: Vec<char> = t.chars().collect();
    chars.iter().enumerate().any(|(i, &c)| {
        c == '@'
            && i > 0
            && chars[i - 1].is_alphanumeric()
            && chars.get(i + 1).is_some_and(|n| n.is_alphanumeric())
    })
}

/// Tek boşluk ya da tireyle ayrılmış rakam dizileri birleştirilir; telefon,
/// kimlik ya da hesap numarasına benzeyen dizi aranır.
fn has_number(t: &str) -> bool {
    let mut runs: Vec<String> = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = t.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_ascii_digit() {
            cur.push(c);
            continue;
        }
        let joins = matches!(c, ' ' | '-')
            && !cur.is_empty()
            && chars.get(i + 1).is_some_and(char::is_ascii_digit);
        if !joins && !cur.is_empty() {
            runs.push(std::mem::take(&mut cur));
        }
    }
    runs.push(cur);
    runs.iter().any(|r| match r.len() {
        10 | 11 => r.starts_with('0') || r.starts_with('5') || is_valid_tckn(r) || is_valid_vkn(r),
        n => n >= 12,
    }) || t.contains("TR")
        && t.split("TR")
            .skip(1)
            .any(|s| s.chars().take(2).all(|c| c.is_ascii_digit()) && s.len() >= 2)
}

/// Listelerdeki, sözcük anlamı olmayan bir ad büyük harfle geçiyor mu?
fn has_name(t: &str) -> bool {
    t.split(|c: char| !c.is_alphabetic())
        .filter(|w| w.chars().next().is_some_and(char::is_uppercase))
        .any(|w| is_listed_name(w) && !is_ambiguous_name(w) && !COMMON_WORDS.contains(&w))
}

/// Soyadı listesinde olan ama cümle başında sıradan sözcük de olabilenler.
const COMMON_WORDS: &[&str] = &[
    "Kara", "Can", "Gök", "Öz", "Tan", "Bulut", "Toprak", "Kartal", "Aslan", "Temiz", "Uzun",
    "Savaş", "Yurt", "Şen", "Tatlı", "Esen", "Güven", "Ateş", "Taş", "Sarı", "Işık", "Gül",
    "Duman", "Bayram", "Güneş", "Yıldız", "Doğan", "Acar", "Keskin", "Ünal", "Kurt", "Koç",
    "Özgür", "Erdem", "Okur", "Selvi",
];

#[cfg(test)]
mod tests {
    use super::*;

    fn text(t: &str) -> Vec<Piece> {
        vec![Piece::Text(t.into())]
    }

    #[test]
    fn finds_leaks() {
        assert_eq!(leak(&text("bana ayse@ornek.com yaz")), Some("ham-eposta"));
        assert_eq!(leak(&text("ara 0532 123 45 67")), Some("ham-numara"));
        assert_eq!(leak(&text("ara 0532-123-45-67")), Some("ham-numara"));
        assert_eq!(leak(&text("TCKN 10000000146")), Some("ham-numara"));
        assert_eq!(leak(&text("IBAN TR33 0006")), Some("ham-numara"));
        assert_eq!(leak(&text("Ahmet bey aradı")), Some("ham-ad"));
        assert_eq!(leak(&text("Yılmaz ailesi")), Some("ham-ad"));
    }

    #[test]
    fn ignores_ordinary_text() {
        for t in [
            "2024-03-12 10:22:31 INFO giriş başarılı",
            "Sipariş 12345 kargoya verildi, 3 gün içinde gelir.",
            "Umut var. Deniz kenarındayız. Can sıkıcı bir gün.",
            "Toplam 1.250,00 TL ödendi.",
            "@ işaretini kullan",
        ] {
            assert_eq!(leak(&text(t)), None, "{t}");
        }
        assert_eq!(leak(&[Piece::Slot("KİŞİ_1".into())]), None);
    }
}
