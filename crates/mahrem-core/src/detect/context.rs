//! Bağlam sözcükleri: numaranın yakınındaki "vergi", "tel", "kart" gibi
//! sözcükler skoru yükseltir.

use crate::Label;

/// Öncesinde bakılan sözcük sayısı.
const BEFORE: usize = 5;
/// Sonrasında bakılan sözcük sayısı.
const AFTER: usize = 3;

/// Sözcüğü küçültüp Türkçe harfleri ASCII'ye indirger.
fn fold(word: &[char]) -> String {
    word.iter()
        .map(|&c| match c {
            'İ' | 'I' | 'ı' => 'i',
            'Ş' | 'ş' => 's',
            'Ğ' | 'ğ' => 'g',
            'Ü' | 'ü' => 'u',
            'Ö' | 'ö' => 'o',
            'Ç' | 'ç' => 'c',
            c => c.to_ascii_lowercase(),
        })
        .collect()
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '.'
}

/// `chars[..at]`'in son `n` sözcüğü.
fn words_before(chars: &[char], at: usize, n: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = at;
    while out.len() < n && i > 0 {
        while i > 0 && !is_word_char(chars[i - 1]) {
            i -= 1;
        }
        let end = i;
        while i > 0 && is_word_char(chars[i - 1]) {
            i -= 1;
        }
        if end > i {
            out.push(fold(&chars[i..end]));
        }
    }
    out
}

/// `chars[at..]`'in ilk `n` sözcüğü.
fn words_after(chars: &[char], at: usize, n: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = at;
    while out.len() < n && i < chars.len() {
        while i < chars.len() && !is_word_char(chars[i]) {
            i += 1;
        }
        let start = i;
        while i < chars.len() && is_word_char(chars[i]) {
            i += 1;
        }
        if i > start {
            out.push(fold(&chars[start..i]));
        }
    }
    out
}

/// Bir etiketin bağlam sözcüklerinin başları. Sözcük bu öneklerden biriyle
/// başlarsa bağlam sayılır ("telefonum", "kimliğim", "vergi").
fn stems(label: Label) -> &'static [&'static str] {
    match label {
        Label::Tckn | Label::Ykn => &[
            "tc", "t.c", "tckn", "kimlik", "ykn", "yabanci", "nufus", "identity",
        ],
        Label::Vkn => &["vergi", "vkn", "v.k.n", "v.d", "vd", "tax"],
        Label::CardNumber => &[
            "kart", "kredi", "banka", "visa", "master", "troy", "amex", "card",
        ],
        Label::Phone => &[
            "tel", "telefon", "cep", "gsm", "numara", "aray", "whatsapp", "faks", "fax", "phone",
            "mobile", "hatt",
        ],
        _ => &[],
    }
}

/// `[start, end)` aralığının çevresinde etiketin bağlam sözcüğü var mı?
pub(crate) fn has_context(chars: &[char], start: usize, end: usize, label: Label) -> bool {
    let stems = stems(label);
    words_before(chars, start, BEFORE)
        .iter()
        .chain(words_after(chars, end, AFTER).iter())
        .any(|w| {
            let w = w.trim_end_matches('.');
            stems.iter().any(|s| w.starts_with(s.trim_end_matches('.'))) && !w.is_empty()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(text: &str, needle: &str, label: Label) -> bool {
        let chars: Vec<char> = text.chars().collect();
        let start = text[..text.find(needle).unwrap()].chars().count();
        has_context(&chars, start, start + needle.chars().count(), label)
    }

    #[test]
    fn finds_context_on_both_sides() {
        assert!(ctx("Vergi no: 1234567890", "1234567890", Label::Vkn));
        assert!(ctx(
            "1234567890 vergi numarasıyla",
            "1234567890",
            Label::Vkn
        ));
        assert!(ctx(
            "T.C. Kimlik No: 10000000146",
            "10000000146",
            Label::Tckn
        ));
        assert!(ctx("TELEFONUM 0532", "0532", Label::Phone));
        assert!(!ctx(
            "Sipariş numarası 1234567890",
            "1234567890",
            Label::Vkn
        ));
    }
}
