//! Türkçe büyük/küçük harf dönüşümü.
//!
//! Unicode'un varsayılan dönüşümü Türkçeye uymaz: `İ` küçültülünce `i̇`
//! (i + birleşen nokta) olur, `I` ise `i` olur. Bu modül `İ` ↔ `i` ve
//! `I` ↔ `ı` eşleşmelerini açıkça işler; diğer harfler için Unicode
//! dönüşümünü kullanır.

/// Metni Türkçe kurallarla küçük harfe çevirir.
///
/// ```
/// use mahrem_core::to_lowercase_tr;
/// assert_eq!(to_lowercase_tr("İSTANBUL ILGAZ"), "istanbul ılgaz");
/// ```
#[must_use]
pub fn to_lowercase_tr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            'İ' => out.push('i'),
            'I' => out.push('ı'),
            _ => out.extend(c.to_lowercase()),
        }
    }
    out
}

/// Metni Türkçe kurallarla büyük harfe çevirir.
///
/// ```
/// use mahrem_core::to_uppercase_tr;
/// assert_eq!(to_uppercase_tr("istanbul ılgaz"), "İSTANBUL ILGAZ");
/// ```
#[must_use]
pub fn to_uppercase_tr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            'i' => out.push('İ'),
            'ı' => out.push('I'),
            _ => out.extend(c.to_uppercase()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dotted_and_dotless_i() {
        assert_eq!(to_lowercase_tr("İIiı"), "iıiı");
        assert_eq!(to_uppercase_tr("İIiı"), "İIİI");
    }

    #[test]
    fn other_turkish_letters() {
        assert_eq!(to_lowercase_tr("ÇĞÖŞÜ"), "çğöşü");
        assert_eq!(to_uppercase_tr("çğöşü"), "ÇĞÖŞÜ");
    }

    #[test]
    fn lowercase_never_emits_combining_dot() {
        assert!(!to_lowercase_tr("İZMİR").contains('\u{0307}'));
    }
}
