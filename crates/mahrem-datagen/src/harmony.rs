//! Ek uyumu denetimi: LLM'in yer tutucuya eklediği ek, yerleştirilecek değere
//! uyuyor mu?
//!
//! Ek yeniden çekilmez; uymayan değer yerine uyan bir değer seçilir. Ekin
//! yalnızca ilk hecesine bakılır: ünlü uyumu (iki ve dört yönlü), kaynaştırma
//! harfi (`n`, `y`, `s`), ünsüz benzeşmesi (`d`/`t`, `c`/`ç`) ve "ile" eki
//! (`'le` / `'yle`). Rakamla biten değerlerde son sayının okunuşuna, büyük
//! harf kısaltmalarda son harfin adına bakılır. Asıl ek çekimi v0.5'te
//! çekirdeğe gelir.

use mahrem_core::fake::number_words;
use mahrem_core::to_lowercase_tr;

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'ı' | 'i' | 'o' | 'ö' | 'u' | 'ü')
}

fn is_front(c: char) -> bool {
    matches!(c, 'e' | 'i' | 'ö' | 'ü')
}

fn is_voiceless(c: char) -> bool {
    matches!(c, 'f' | 's' | 't' | 'k' | 'ç' | 'ş' | 'h' | 'p')
}

fn four_way(last: char) -> char {
    match last {
        'a' | 'ı' => 'ı',
        'e' | 'i' => 'i',
        'o' | 'u' => 'u',
        _ => 'ü',
    }
}

fn letter_name(c: char) -> &'static str {
    match c {
        'A' => "a",
        'B' => "be",
        'C' => "ce",
        'Ç' => "çe",
        'D' => "de",
        'E' => "e",
        'F' => "fe",
        'G' => "ge",
        'Ğ' => "yumuşak ge",
        'H' => "he",
        'I' => "ı",
        'İ' => "i",
        'J' => "je",
        'K' => "ka",
        'L' => "le",
        'M' => "me",
        'N' => "ne",
        'O' => "o",
        'Ö' => "ö",
        'P' => "pe",
        'R' => "re",
        'S' => "se",
        'Ş' => "şe",
        'T' => "te",
        'U' => "u",
        'Ü' => "ü",
        'V' => "ve",
        'Y' => "ye",
        'Z' => "ze",
        'Q' => "kü",
        'W' => "dabılyu",
        'X' => "iks",
        _ => "",
    }
}

/// Değerin olası okunuşlarının sonu, küçük harf. Kısaltmalar hem harf harf
/// ("ha-ı-ve") hem sözcük gibi ("hiv") okunabilir; ikisi de döner.
fn spoken_tails(value: &str) -> Vec<String> {
    let trimmed = value.trim_end_matches(|c: char| !c.is_alphanumeric());
    let token: String = {
        let mut t: Vec<char> = trimmed
            .chars()
            .rev()
            .take_while(|c| c.is_alphanumeric())
            .collect();
        t.reverse();
        t.into_iter().collect()
    };
    if token.ends_with(|c: char| c.is_ascii_digit()) {
        let digits: String = {
            let mut d: Vec<char> = token
                .chars()
                .rev()
                .take_while(char::is_ascii_digit)
                .take(18)
                .collect();
            d.reverse();
            d.into_iter().collect()
        };
        return vec![number_words(digits.parse().unwrap_or(0))];
    }
    let letters = token.chars().filter(|c| c.is_alphabetic()).count();
    let acronym = token
        .chars()
        .all(|c| c.is_uppercase() || c.is_ascii_digit())
        && (letters >= 2 || token.chars().count() == 1);
    if acronym {
        // Kısaltmalar çoğunlukla Latin kökenli okunur: "HIV" → "hiv".
        let word = token.to_lowercase();
        let letter = token
            .chars()
            .last()
            .map(letter_name)
            .unwrap_or_default()
            .to_owned();
        if word.chars().any(is_vowel) {
            return vec![letter, word];
        }
        return vec![letter];
    }
    vec![to_lowercase_tr(&token)]
}

/// Ek değere uyuyor mu? Boş ek her değere uyar.
#[must_use]
pub fn fits(value: &str, suffix: &str) -> bool {
    let suffix = to_lowercase_tr(suffix);
    let mut s = suffix.chars();
    let Some(s0) = s.next() else { return true };
    let s1 = s.next();
    spoken_tails(value)
        .iter()
        .any(|spoken| fits_spoken(spoken, &suffix, s0, s1))
}

fn fits_spoken(spoken: &str, suffix: &str, s0: char, s1: Option<char>) -> bool {
    let Some(last_vowel) = spoken.chars().rev().find(|&c| is_vowel(c)) else {
        return true;
    };
    let Some(last) = spoken.chars().last() else {
        return true;
    };
    let ends_vowel = is_vowel(last);
    if let Some(sv) = suffix.chars().find(|&c| is_vowel(c)) {
        let ok = match sv {
            'a' | 'e' => is_front(sv) == is_front(last_vowel),
            'ı' | 'i' | 'u' | 'ü' => sv == four_way(last_vowel),
            _ => true,
        };
        if !ok {
            return false;
        }
    }
    let next_is_vowel = s1.is_some_and(is_vowel);
    if is_vowel(s0) && ends_vowel {
        return false;
    }
    if matches!(s0, 'n' | 'y') && next_is_vowel && !ends_vowel {
        return false;
    }
    if s0 == 'y' && s1 == Some('l') && !ends_vowel {
        return false;
    }
    if s0 == 's' && s1.is_some_and(|c| matches!(c, 'ı' | 'i' | 'u' | 'ü')) && !ends_vowel {
        return false;
    }
    if s0 == 'l' && ends_vowel {
        return false;
    }
    if matches!(s0, 'd' | 'c') && is_voiceless(last) {
        return false;
    }
    if matches!(s0, 't' | 'ç') && !is_voiceless(last) {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_words() {
        for (v, s, ok) in [
            ("Ahmet", "in", true),
            ("Ahmet", "nin", false),
            ("Ayşe", "nin", true),
            ("Ayşe", "in", false),
            ("Ayşe", "nın", false),
            ("Ahmet", "te", true),
            ("Ahmet", "de", false),
            ("Ankara", "da", true),
            ("İzmir", "de", true),
            ("İzmir", "da", false),
            ("Ali", "yle", true),
            ("Ahmet", "le", true),
            ("Ayşe", "le", false),
            ("Ahmet", "yle", false),
            ("Ahmet", "si", false),
            ("Ayşe", "si", true),
            ("Mehmet Kaya", "nın", true),
            ("Mehmet Kaya", "ın", false),
            ("Kaya Lojistik Ltd. Şti.", "ye", true),
            ("diyabet", "im", true),
            ("diyabet", "ım", false),
            ("Ahmet", "", true),
            ("Murat", "tan", true),
            ("Ömür", "ün", true),
            ("Ömür", "un", false),
        ] {
            assert_eq!(fits(v, s), ok, "{v}'{s}");
        }
    }

    #[test]
    fn numbers_and_acronyms() {
        for (v, s, ok) in [
            ("0532 123 45 67", "ye", true),
            ("0532 123 45 67", "e", false),
            ("5", "te", true),
            ("5", "de", false),
            ("6", "da", true),
            ("6", "ya", true),
            ("10", "a", true),
            ("10", "dan", true),
            ("MS", "ten", false),
            ("MS", "ye", true),
            ("Kaya Gıda A.Ş.", "ye", true),
            ("HIV", "e", true),
            ("34 ABC 123", "ü", true),
            ("ayse@gmail.com", "a", true),
            ("ayse@gmail.com", "e", false),
        ] {
            assert_eq!(fits(v, s), ok, "{v}'{s}");
        }
    }
}
