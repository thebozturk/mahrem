//! Numaralar, kodlar ve tarihler.

use super::Rng;
use super::data::{MONTHS, PROVINCES};

fn to_digits(s: &str) -> Vec<u32> {
    s.bytes().map(|b| u32::from(b - b'0')).collect()
}

fn tc_checksum(first9: &str) -> String {
    let d = to_digits(first9);
    let odd: u32 = d.iter().step_by(2).sum();
    let even: u32 = d[1..].iter().step_by(2).sum();
    let tenth = (odd * 7 + 90 - even) % 10;
    let eleventh = (d.iter().sum::<u32>() + tenth) % 10;
    format!("{first9}{tenth}{eleventh}")
}

/// T.C. kimlik numarası: 11 hane, checksum'ı doğru. "99" ile başlamaz
/// (o önek yabancı kimlik numarasınındır).
pub fn tckn(rng: &mut Rng) -> String {
    loop {
        let first = rng.digit_in(1, 9);
        let n = tc_checksum(&format!("{first}{}", rng.digits(8)));
        if !n.starts_with("99") {
            return n;
        }
    }
}

/// Yabancı kimlik numarası: "99" ile başlar, T.C. kimlik algoritmasından geçer.
pub fn ykn(rng: &mut Rng) -> String {
    tc_checksum(&format!("99{}", rng.digits(7)))
}

/// Vergi kimlik numarası: 10 hane, checksum'ı doğru.
pub fn vkn(rng: &mut Rng) -> String {
    let first9 = rng.digits(9);
    let mut sum = 0;
    for (digit, weight) in to_digits(&first9).into_iter().zip((1..=9).rev()) {
        let a = (digit + weight) % 10;
        if a != 0 {
            let b = (a * 2_u32.pow(weight)) % 9;
            sum += if b == 0 { 9 } else { b };
        }
    }
    format!("{first9}{}", (10 - sum % 10) % 10)
}

fn group(s: &str, size: usize) -> String {
    s.as_bytes()
        .chunks(size)
        .map(|c| std::str::from_utf8(c).expect("ASCII"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Türkiye IBAN'ı: bitişik ya da dörderli gruplar halinde.
pub fn iban_tr(rng: &mut Rng) -> String {
    // Banka kodu (5) + rezerv hane (0) + hesap numarası (16).
    let bban = format!("{}0{}", rng.digits(5), rng.digits(16));
    // Kontrol hanesi: BBAN + "TR00" sayıya çevrilir ("T" = 29, "R" = 27), 98 − mod 97.
    let mut rem: u32 = 0;
    for c in bban.bytes().chain(*b"292700") {
        rem = (rem * 10 + u32::from(c - b'0')) % 97;
    }
    let iban = format!("TR{:02}{bban}", 98 - rem);
    if rng.chance(50) {
        group(&iban, 4)
    } else {
        iban
    }
}

/// Ödeme kartı numarası: 16 hane, Luhn'dan geçer; bitişik ya da dörderli.
pub fn card_number(rng: &mut Rng) -> String {
    // Visa, Mastercard ve Troy'a benzeyen önekler.
    let prefix = *rng.pick(&["4", "51", "52", "53", "54", "55", "9792"]);
    let mut body = format!("{prefix}{}", rng.digits(15 - prefix.len()));
    let sum: u32 = to_digits(&body)
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &x)| {
            if i % 2 == 0 {
                let y = x * 2;
                if y > 9 { y - 9 } else { y }
            } else {
                x
            }
        })
        .sum();
    body.push_str(&((10 - sum % 10) % 10).to_string());
    if rng.chance(60) {
        group(&body, 4)
    } else {
        body
    }
}

/// Telefon numarası: çoğunlukla cep, kimi zaman sabit hat ya da kurumsal
/// (444, 0850); yaygın yazım biçimlerinden biriyle.
pub fn phone(rng: &mut Rng) -> String {
    match rng.below(10) {
        0 => {
            let area = *rng.pick(&[
                "212", "216", "312", "232", "224", "242", "322", "262", "332", "342",
            ]);
            format!(
                "0{area} {} {} {}",
                rng.digits(3),
                rng.digits(2),
                rng.digits(2)
            )
        }
        1 => {
            if rng.chance(50) {
                format!("444 {} {}", rng.digits(2), rng.digits(2))
            } else {
                format!("0850 {} {} {}", rng.digits(3), rng.digits(2), rng.digits(2))
            }
        }
        _ => {
            let operator = format!(
                "5{}",
                rng.pick(&[
                    "01", "05", "06", "07", "30", "31", "32", "33", "34", "35", "36", "37", "38",
                    "39", "40", "41", "42", "43", "44", "45", "46", "47", "48", "49", "50", "51",
                    "52", "53", "54", "55", "59",
                ])
            );
            let (a, b, c) = (rng.digits(3), rng.digits(2), rng.digits(2));
            match rng.below(6) {
                0 => format!("0{operator}{a}{b}{c}"),
                1 => format!("+90 {operator} {a} {b} {c}"),
                2 => format!("0 ({operator}) {a} {b} {c}"),
                3 => format!("0{operator}-{a}-{b}-{c}"),
                _ => format!("0{operator} {a} {b} {c}"),
            }
        }
    }
}

/// Pasaport numarası: "U" ve 8 hane.
pub fn passport(rng: &mut Rng) -> String {
    format!("U{}", rng.digits(8))
}

/// Sürücü belgesi numarası: 6 hane.
pub fn driver_license(rng: &mut Rng) -> String {
    format!("{}{}", rng.range(1, 9), rng.digits(5))
}

/// SGK sicil numarası: 13 hane.
pub fn sgk_number(rng: &mut Rng) -> String {
    format!("{}{}", rng.range(1, 9), rng.digits(12))
}

/// MERSİS numarası: "0" ile başlayan 16 hane.
pub fn mersis(rng: &mut Rng) -> String {
    format!("0{}", rng.digits(15))
}

/// Kuruma özgü kişi numarası: müşteri, abone, poliçe, hasta numarası.
pub fn account_id(rng: &mut Rng) -> String {
    let letters =
        |rng: &mut Rng, n: usize| -> String { (0..n).map(|_| rng.upper_letter()).collect() };
    match rng.below(4) {
        0 => format!("{}{}", rng.range(1, 9), {
            let n = rng.size(6, 9);
            rng.digits(n)
        }),
        1 => {
            let l = letters(rng, 2);
            format!("{l}{}", rng.digits(7))
        }
        2 => {
            let l = letters(rng, 3);
            format!("{l}-{}", rng.digits(6))
        }
        _ => format!("{}-{}", rng.digits(4), rng.digits(6)),
    }
}

/// Araç plakası: `34 ABC 123` biçiminde.
pub fn license_plate(rng: &mut Rng) -> String {
    // Plakalarda Türkçeye özgü harfler ile Q, W, X kullanılmaz.
    const LETTERS: &[u8] = b"ABCDEFGHIJKLMNOPRSTUVYZ";
    let (_, code) = *rng.pick(PROVINCES);
    let n_letters = rng.size(1, 3);
    let letters: String = (0..n_letters)
        .map(|_| char::from(*rng.pick(LETTERS)))
        .collect();
    let n_digits = match n_letters {
        1 => 4,
        2 => rng.size(3, 4),
        _ => rng.size(2, 3),
    };
    let digits = format!("{}{}", rng.range(1, 9), rng.digits(n_digits - 1));
    if rng.chance(85) {
        format!("{code:02} {letters} {digits}")
    } else {
        format!("{code:02}{letters}{digits}")
    }
}

/// Cihaz seri numarası ya da IMEI.
pub fn serial_number(rng: &mut Rng) -> String {
    const ALNUM: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ0123456789";
    if rng.chance(40) {
        // IMEI: 15 hane, son hane Luhn.
        let body = format!("35{}", rng.digits(12));
        let sum: u32 = to_digits(&body)
            .iter()
            .rev()
            .enumerate()
            .map(|(i, &x)| {
                if i % 2 == 0 {
                    let y = x * 2;
                    if y > 9 { y - 9 } else { y }
                } else {
                    x
                }
            })
            .sum();
        return format!("{body}{}", (10 - sum % 10) % 10);
    }
    let n = rng.size(10, 12);
    (0..n).map(|_| char::from(*rng.pick(ALNUM))).collect()
}

/// IP adresi: çoğunlukla IPv4 (yerel ağ adresleri dahil), kimi zaman IPv6.
pub fn ip_address(rng: &mut Rng) -> String {
    match rng.below(10) {
        0..=1 => format!("192.168.{}.{}", rng.range(0, 255), rng.range(1, 254)),
        2 => format!(
            "10.{}.{}.{}",
            rng.range(0, 255),
            rng.range(0, 255),
            rng.range(1, 254)
        ),
        3 => {
            let groups: Vec<String> = (0..4)
                .map(|_| format!("{:x}", rng.below(0x1_0000)))
                .collect();
            format!("2a02:{}::{}", groups[..3].join(":"), groups[3])
        }
        _ => {
            // Ayrılmış aralıklar dışında bir ilk oktet.
            let mut first = rng.range(1, 223);
            while [10, 127, 169, 172, 192].contains(&first) {
                first = rng.range(1, 223);
            }
            format!(
                "{first}.{}.{}.{}",
                rng.range(0, 255),
                rng.range(0, 255),
                rng.range(1, 254)
            )
        }
    }
}

/// MAC adresi: `3C:22:FB:1A:9E:04` ya da tireli, küçük harfli biçimler.
pub fn mac_address(rng: &mut Rng) -> String {
    let bytes: Vec<String> = (0..6).map(|_| format!("{:02X}", rng.below(256))).collect();
    match rng.below(3) {
        0 => bytes.join(":"),
        1 => bytes.join("-"),
        _ => bytes.join(":").to_lowercase(),
    }
}

/// Doğum tarihi: `12.03.1990`, `12/03/1990`, `12 Mart 1990` ya da yalnızca yıl.
pub fn birth_date(rng: &mut Rng) -> String {
    let year = rng.range(1940, 2024);
    let month = rng.size(1, 12);
    let max_day = match month {
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    let day = rng.range(1, max_day);
    let month_name = MONTHS[month - 1];
    match rng.below(10) {
        0..=3 => format!("{day:02}.{month:02}.{year}"),
        4 => format!("{day:02}/{month:02}/{year}"),
        5 => format!("{year}-{month:02}-{day:02}"),
        6..=8 => format!("{day} {month_name} {year}"),
        _ => year.to_string(),
    }
}

/// Yaş, rakamla. `child` ise 0–17, değilse 18–90.
pub fn age(rng: &mut Rng, child: bool) -> String {
    if child {
        rng.range(0, 17).to_string()
    } else {
        rng.range(18, 90).to_string()
    }
}

/// Gizli bilgi: API anahtarı, erişim belirteci, parola, PIN, CVV ya da
/// tek kullanımlık kod.
pub fn secret(rng: &mut Rng) -> String {
    const B62: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    const B64URL: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let chars = |rng: &mut Rng, alphabet: &[u8], n: usize| -> String {
        (0..n).map(|_| char::from(*rng.pick(alphabet))).collect()
    };
    match rng.below(8) {
        0 => format!("sk-{}", chars(rng, B62, 40)),
        1 => format!("ghp_{}", chars(rng, B62, 36)),
        2 => {
            let payload = chars(rng, B64URL, 60);
            let sig = chars(rng, B64URL, 43);
            format!("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.{payload}.{sig}")
        }
        3 | 4 => {
            let word = *rng.pick(&[
                "Kedi", "Deniz", "Bahar", "Yildiz", "Gunes", "Ornek", "Sifre", "Kartal",
            ]);
            let sym = *rng.pick(&["!", ".", "*", "?", "#", ""]);
            format!("{word}{sym}{}", {
                let n = rng.size(2, 4);
                rng.digits(n)
            })
        }
        5 => {
            let n = rng.size(4, 6);
            rng.digits(n)
        }
        6 => rng.digits(3),
        _ => rng.digits(6),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats() {
        for seed in 0..1000 {
            let mut rng = Rng::new(seed);
            let m = mersis(&mut rng);
            assert!(m.len() == 16 && m.starts_with('0'), "{m}");
            let p = passport(&mut rng);
            assert!(p.len() == 9 && p.starts_with('U'), "{p}");
            let phone = phone(&mut rng);
            let digits = phone.chars().filter(char::is_ascii_digit).count();
            assert!([7, 11, 12].contains(&digits), "{phone}");
            let plate = license_plate(&mut rng);
            let code: u8 = plate[..2].parse().unwrap();
            assert!((1..=81).contains(&code), "{plate}");
            assert!(
                !plate.contains(['Ç', 'Ğ', 'İ', 'Ö', 'Ş', 'Ü', 'Q', 'W', 'X']),
                "{plate}"
            );
            let mac = mac_address(&mut rng);
            assert_eq!(mac.len(), 17, "{mac}");
            let a: u64 = age(&mut rng, true).parse().unwrap();
            assert!(a < 18);
            let a: u64 = age(&mut rng, false).parse().unwrap();
            assert!((18..=90).contains(&a));
            let d = birth_date(&mut rng);
            assert!(d.chars().any(|c| c.is_ascii_digit()), "{d}");
        }
    }

    #[test]
    fn imei_passes_luhn() {
        for seed in 0..500 {
            let s = serial_number(&mut Rng::new(seed));
            if s.len() == 15 && s.bytes().all(|b| b.is_ascii_digit()) {
                assert!(crate::validators::is_valid_card_number(&s), "{s}");
            }
        }
    }
}
