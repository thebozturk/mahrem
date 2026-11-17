//! Checksum doğrulayıcıları.
//!
//! Fonksiyonlar yalnızca sıkıştırılmış biçimi kabul eder: boşluk, tire ya da
//! başka ayraç içeren girdiler geçersizdir. Düzensiz yazımın normalleştirilmesi
//! tespit katmanının işidir.

/// Girdi yalnızca ASCII rakamlardan oluşuyorsa rakam değerlerini döner.
fn digits(s: &str) -> Option<Vec<u32>> {
    s.bytes()
        .map(|b| b.is_ascii_digit().then(|| u32::from(b - b'0')))
        .collect()
}

/// T.C. kimlik numarasını doğrular.
///
/// 11 hane, ilk hane 0 olamaz. 10. hane: tek konumların toplamının 7 katı ile
/// 90'ın toplamından çift konumların toplamı çıkarılır, mod 10 alınır.
/// 11. hane: ilk 10 hanenin toplamı mod 10.
///
/// ```
/// use mahrem_core::is_valid_tckn;
/// assert!(is_valid_tckn("10000000146"));
/// assert!(!is_valid_tckn("10000000145"));
/// ```
#[must_use]
pub fn is_valid_tckn(s: &str) -> bool {
    let Some(d) = digits(s) else {
        return false;
    };
    if d.len() != 11 || d[0] == 0 {
        return false;
    }
    let odd: u32 = d[0..9].iter().step_by(2).sum();
    let even: u32 = d[1..8].iter().step_by(2).sum();
    let tenth = (odd * 7 + 90 - even) % 10;
    let eleventh = (d[..10].iter().sum::<u32>()) % 10;
    d[9] == tenth && d[10] == eleventh
}

/// Yabancı kimlik numarasını doğrular: "99" ile başlar ve T.C. kimlik
/// numarası algoritmasından geçer.
///
/// ```
/// use mahrem_core::is_valid_ykn;
/// assert!(is_valid_ykn("99000000042"));
/// assert!(!is_valid_ykn("10000000146"));
/// ```
#[must_use]
pub fn is_valid_ykn(s: &str) -> bool {
    s.starts_with("99") && is_valid_tckn(s)
}

/// Vergi kimlik numarasını doğrular.
///
/// 10 hane. İlk 9 hanenin her biri için ağırlık `9 − i` ve
/// `a = (rakam + ağırlık) mod 10`. `a ≠ 0` ise `b = (a · 2^ağırlık) mod 9`;
/// `b` 0 çıkarsa 9 alınır. Kontrol hanesi `(10 − toplam mod 10) mod 10`.
///
/// ```
/// use mahrem_core::is_valid_vkn;
/// assert!(is_valid_vkn("1234567890"));
/// assert!(!is_valid_vkn("1234567891"));
/// ```
#[must_use]
pub fn is_valid_vkn(s: &str) -> bool {
    let Some(d) = digits(s) else {
        return false;
    };
    if d.len() != 10 {
        return false;
    }
    let mut sum = 0;
    for (i, &digit) in d[..9].iter().enumerate() {
        let weight = 9 - u32::try_from(i).unwrap_or(0);
        let a = (digit + weight) % 10;
        if a != 0 {
            let b = (a * 2_u32.pow(weight)) % 9;
            sum += if b == 0 { 9 } else { b };
        }
    }
    d[9] == (10 - sum % 10) % 10
}

/// Türkiye IBAN'ını doğrular.
///
/// 26 karakter, "TR" ile başlar. 2–9. konumlar rakam (kontrol hanesi, banka
/// kodu, rezerv hane), 10–25. konumlar büyük harf ya da rakam. Mod 97 sonucu 1
/// olmalıdır.
///
/// ```
/// use mahrem_core::is_valid_iban_tr;
/// assert!(is_valid_iban_tr("TR330006100519786457841326"));
/// assert!(!is_valid_iban_tr("TR330006100519786457841327"));
/// ```
#[must_use]
pub fn is_valid_iban_tr(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 26 || &b[..2] != b"TR" {
        return false;
    }
    if !b[2..10].iter().all(u8::is_ascii_digit) {
        return false;
    }
    if !b[10..]
        .iter()
        .all(|c| c.is_ascii_digit() || c.is_ascii_uppercase())
    {
        return false;
    }
    let mut rem: u32 = 0;
    for &c in b[4..].iter().chain(&b[..4]) {
        if c.is_ascii_digit() {
            rem = (rem * 10 + u32::from(c - b'0')) % 97;
        } else {
            rem = (rem * 100 + u32::from(c - b'A') + 10) % 97;
        }
    }
    rem == 1
}

/// Kart numarasını Luhn algoritmasıyla doğrular. 12–19 hane kabul edilir.
///
/// ```
/// use mahrem_core::is_valid_card_number;
/// assert!(is_valid_card_number("4111111111111111"));
/// assert!(!is_valid_card_number("4111111111111112"));
/// ```
#[must_use]
pub fn is_valid_card_number(s: &str) -> bool {
    let Some(d) = digits(s) else {
        return false;
    };
    if !(12..=19).contains(&d.len()) {
        return false;
    }
    let sum: u32 = d
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &x)| {
            if i % 2 == 1 {
                let y = x * 2;
                if y > 9 { y - 9 } else { y }
            } else {
                x
            }
        })
        .sum();
    sum.is_multiple_of(10)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn tckn_from_prefix(p: &[u32; 9]) -> String {
        let odd: u32 = p.iter().step_by(2).sum();
        let even: u32 = p[1..].iter().step_by(2).sum();
        let tenth = (odd * 7 + 90 - even) % 10;
        let eleventh = (p.iter().sum::<u32>() + tenth) % 10;
        p.iter()
            .chain([tenth, eleventh].iter())
            .map(|d| char::from_digit(*d, 10).unwrap_or('x'))
            .collect()
    }

    #[test]
    fn tckn_rejects_leading_zero_and_bad_length() {
        assert!(!is_valid_tckn("01234567890"));
        assert!(!is_valid_tckn("1000000014"));
        assert!(!is_valid_tckn("100000001460"));
        assert!(!is_valid_tckn(""));
    }

    #[test]
    fn tckn_rejects_separators() {
        assert!(!is_valid_tckn("100 000 001 46"));
        assert!(!is_valid_tckn("1000000014६"));
    }

    #[test]
    fn ykn_requires_prefix() {
        assert!(is_valid_ykn("99111111194"));
        assert!(!is_valid_ykn("98000000042"));
    }

    #[test]
    fn vkn_rejects_bad_length() {
        assert!(!is_valid_vkn("123456789"));
        assert!(!is_valid_vkn("12345678901"));
    }

    #[test]
    fn iban_rejects_lowercase_and_spaces() {
        assert!(!is_valid_iban_tr("tr330006100519786457841326"));
        assert!(!is_valid_iban_tr("TR33 0006 1005 1978 6457 8413 26"));
    }

    #[test]
    fn card_length_bounds() {
        assert!(!is_valid_card_number("42"));
        assert!(!is_valid_card_number("41111111111111111111"));
    }

    proptest! {
        #[test]
        fn validators_never_panic(s in "\\PC*") {
            let _ = is_valid_tckn(&s);
            let _ = is_valid_ykn(&s);
            let _ = is_valid_vkn(&s);
            let _ = is_valid_iban_tr(&s);
            let _ = is_valid_card_number(&s);
        }

        #[test]
        fn generated_tckn_is_valid(first in 1u32..10, rest in proptest::array::uniform8(0u32..10)) {
            let mut p = [0u32; 9];
            p[0] = first;
            p[1..].copy_from_slice(&rest);
            prop_assert!(is_valid_tckn(&tckn_from_prefix(&p)));
        }
    }
}
