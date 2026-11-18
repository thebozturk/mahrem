use super::Rng;
use super::data::{AMBIGUOUS, FEMALE, MAIL_DOMAINS, MALE, SURNAMES, UNISEX};
use crate::case::to_lowercase_tr;

/// Kişinin cinsiyeti; "Bey" ve "Hanım" gibi hitaplarla uyum için.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gender {
    /// Kadın.
    Female,
    /// Erkek.
    Male,
}

/// Bir ad. `gender` verilirse o cinsiyete uygun adlardan (iki cinsiyette de
/// kullanılan adlar dahil) seçer.
pub fn first_name(rng: &mut Rng, gender: Option<Gender>) -> &'static str {
    let gender = gender.unwrap_or(if rng.chance(50) {
        Gender::Female
    } else {
        Gender::Male
    });
    if rng.chance(5) {
        return rng.pick(UNISEX);
    }
    match gender {
        Gender::Female => rng.pick(FEMALE),
        Gender::Male => rng.pick(MALE),
    }
}

/// Bir soyadı.
pub fn surname(rng: &mut Rng) -> &'static str {
    rng.pick(SURNAMES)
}

/// Ad ve soyad; kimi zaman iki adlı ("Mehmet Ali Kaya").
pub fn full_name(rng: &mut Rng, gender: Option<Gender>) -> String {
    let first = first_name(rng, gender);
    let surname = surname(rng);
    if rng.chance(12) {
        let mut second = first_name(rng, gender);
        while second == first {
            second = first_name(rng, gender);
        }
        format!("{first} {second} {surname}")
    } else {
        format!("{first} {surname}")
    }
}

/// Aynı zamanda sıradan bir sözcük olan bir ad ("Umut", "Deniz"); zor durum
/// örnekleri için.
pub fn ambiguous_first_name(rng: &mut Rng) -> &'static str {
    rng.pick(AMBIGUOUS)
}

/// Sözcük listelerdeki bir ad ya da soyad mı? Üretilen metinde yer tutucu
/// dışında kalmış ad aramak için.
#[must_use]
pub fn is_listed_name(word: &str) -> bool {
    FEMALE.contains(&word)
        || MALE.contains(&word)
        || UNISEX.contains(&word)
        || SURNAMES.contains(&word)
}

/// Ad aynı zamanda sıradan bir sözcük mü? ("Umut", "Deniz")
#[must_use]
pub fn is_ambiguous_name(name: &str) -> bool {
    AMBIGUOUS.contains(&name)
}

/// Türkçe harfleri ASCII karşılığına çevirip küçültür: kullanıcı adı ve
/// e-posta için.
fn ascii_lower(s: &str) -> String {
    to_lowercase_tr(s)
        .chars()
        .filter_map(|c| match c {
            'ç' => Some('c'),
            'ğ' => Some('g'),
            'ı' | 'î' => Some('i'),
            'ö' => Some('o'),
            'ş' => Some('s'),
            'ü' | 'û' => Some('u'),
            'â' => Some('a'),
            ' ' => None,
            c => Some(c),
        })
        .collect()
}

fn handle(rng: &mut Rng, name: Option<&str>) -> String {
    let owned;
    let name = if let Some(n) = name {
        n
    } else {
        owned = full_name(rng, None);
        &owned
    };
    let parts: Vec<String> = name.split(' ').map(ascii_lower).collect();
    let first = &parts[0];
    let last = &parts[parts.len() - 1];
    let sep = *rng.pick(&[".", "_", ""]);
    let base = match rng.below(4) {
        0 => format!("{first}{sep}{last}"),
        1 => format!("{}{sep}{last}", &first[..1]),
        2 => format!("{first}{sep}{}", &last[..1]),
        _ => format!("{last}{sep}{first}"),
    };
    if rng.chance(60) {
        format!("{base}{}", rng.range(1, 99))
    } else {
        base
    }
}

/// Kişiyi tanımlayan kullanıcı adı: `@ayse_demir`, `ayse.demir84`.
pub fn username(rng: &mut Rng, name: Option<&str>) -> String {
    let h = handle(rng, name);
    if rng.chance(50) { format!("@{h}") } else { h }
}

/// E-posta adresi. `name` verilirse adres o addan türetilir.
pub fn email(rng: &mut Rng, name: Option<&str>) -> String {
    let local = handle(rng, name);
    format!("{local}@{}", rng.pick(MAIL_DOMAINS))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_respect_gender() {
        for seed in 0..500 {
            let mut rng = Rng::new(seed);
            let f = first_name(&mut rng, Some(Gender::Female));
            assert!(FEMALE.contains(&f) || UNISEX.contains(&f), "{f}");
            let m = first_name(&mut rng, Some(Gender::Male));
            assert!(MALE.contains(&m) || UNISEX.contains(&m), "{m}");
        }
    }

    #[test]
    fn emails_and_usernames_are_ascii() {
        for seed in 0..500 {
            let mut rng = Rng::new(seed);
            let e = email(&mut rng, Some("Çağrı Öztürk"));
            assert!(e.is_ascii() && e.matches('@').count() == 1, "{e}");
            let u = username(&mut rng, None);
            assert!(u.is_ascii() && !u.contains(' '), "{u}");
        }
        assert!(is_ambiguous_name("Umut"));
        assert!(is_ambiguous_name(ambiguous_first_name(&mut Rng::new(1))));
        assert!(is_listed_name("Ayşe") && is_listed_name("Yılmaz") && !is_listed_name("araba"));
        assert!(!is_ambiguous_name("Mehmet"));
    }
}
