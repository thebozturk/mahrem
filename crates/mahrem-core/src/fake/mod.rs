//! Sahte değer üreticileri.
//!
//! Sentetik veri hattı ve ileride "Türkçe sahte veri" operatörü aynı
//! üreticiyi kullanır. Her üretici tohumlu bir [`Rng`] alır; aynı tohum aynı
//! değeri verir.
//!
//! - Kimlik ve finans numaraları checksum'ı doğru üretilir ve
//!   [`crate::validators`]'tan geçer.
//! - Ad, soyad, il, ilçe ve kurum kalıpları bu modülde yazılı listelerden
//!   gelir. Değerler rastgeledir; gerçek bir kişiye ya da hesaba denk gelmesi
//!   tesadüftür ve başka hiçbir bilgiyle ilişkilendirilmez.
//! - Üreticiler yalnızca değeri döner, çevresindeki sözcükleri değil:
//!   `diyabet`, `Tel: 0532 …` değil `0532 …`. Span sınırları etiketleme
//!   yönergesine uyar.
//!
//! ```
//! use mahrem_core::fake::{Rng, generate};
//! use mahrem_core::{Label, is_valid_tckn};
//!
//! let mut rng = Rng::new(42);
//! let tckn = generate(Label::Tckn, &mut rng);
//! assert!(is_valid_tckn(&tckn));
//! assert_eq!(tckn, generate(Label::Tckn, &mut Rng::new(42)));
//! ```

mod data;
mod names;
mod numbers;
mod places;
mod rng;
mod special;
mod words;

pub use names::{Gender, first_name, full_name, surname, username};
pub use numbers::{
    account_id, age, birth_date, card_number, driver_license, iban_tr, ip_address, license_plate,
    mac_address, mersis, passport, phone, secret, serial_number, sgk_number, tckn, vkn, ykn,
};
pub use places::{OrgKind, address, organization};
pub use rng::Rng;
pub use special::special_category;
pub use words::number_words;

pub use names::{ambiguous_first_name, email, is_ambiguous_name, is_listed_name};

use crate::Label;

/// Bir etiket için varsayılan biçimde değer üretir.
///
/// Kişi rastgele cinsiyette tam addır, yaş yetişkin yaşıdır, tarih doğum
/// tarihidir. Daha ayrıntılı seçim için ilgili üreticiyi doğrudan çağırın.
///
/// # Panics
///
/// Olmaz: her özel nitelikli etiketin üreticisi vardır (testte denetlenir).
#[must_use]
pub fn generate(label: Label, rng: &mut Rng) -> String {
    match label {
        Label::Tckn => tckn(rng),
        Label::Ykn => ykn(rng),
        Label::Vkn => vkn(rng),
        Label::Iban => iban_tr(rng),
        Label::CardNumber => card_number(rng),
        Label::Phone => phone(rng),
        Label::Email => email(rng, None),
        Label::Passport => passport(rng),
        Label::DriverLicense => driver_license(rng),
        Label::SgkNumber => sgk_number(rng),
        Label::Mersis => mersis(rng),
        Label::AccountId => account_id(rng),
        Label::LicensePlate => license_plate(rng),
        Label::SerialNumber => serial_number(rng),
        Label::IpAddress => ip_address(rng),
        Label::MacAddress => mac_address(rng),
        Label::Date => birth_date(rng),
        Label::Secret => secret(rng),
        Label::Person => full_name(rng, None),
        Label::Address => address(rng, false),
        Label::Organization => {
            let kind = OrgKind::any(rng);
            organization(rng, kind, false)
        }
        Label::Age => age(rng, false),
        Label::EthnicOrigin
        | Label::PoliticalOpinion
        | Label::Belief
        | Label::Attire
        | Label::Membership
        | Label::Health
        | Label::SexLife
        | Label::CriminalRecord
        | Label::Biometric
        | Label::Genetic => {
            special_category(label, rng).expect("özel nitelikli etiketlerin hepsinin üreticisi var")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validators::{
        is_valid_card_number, is_valid_iban_tr, is_valid_tckn, is_valid_vkn, is_valid_ykn,
    };

    #[test]
    fn every_label_has_a_clean_value() {
        for seed in 0..200 {
            let mut rng = Rng::new(seed);
            for &label in Label::ALL {
                let v = generate(label, &mut rng);
                assert!(!v.is_empty(), "{label}");
                assert_eq!(v.trim(), v, "{label}: {v:?}");
                assert!(!v.contains('\n'), "{label}: {v:?}");
            }
        }
    }

    #[test]
    fn same_seed_same_values() {
        let all = |seed| {
            let mut rng = Rng::new(seed);
            Label::ALL
                .iter()
                .map(|&l| generate(l, &mut rng))
                .collect::<Vec<_>>()
        };
        assert_eq!(all(7), all(7));
        assert_ne!(all(7), all(8));
    }

    #[test]
    fn identity_numbers_pass_validators() {
        let compact = |s: String| s.replace(' ', "");
        for seed in 0..2000 {
            let mut rng = Rng::new(seed);
            let id = tckn(&mut rng);
            assert!(is_valid_tckn(&id) && !id.starts_with("99"), "{id}");
            let foreign = ykn(&mut rng);
            assert!(is_valid_ykn(&foreign), "{foreign}");
            let tax = vkn(&mut rng);
            assert!(is_valid_vkn(&tax), "{tax}");
            let iban = compact(iban_tr(&mut rng));
            assert!(is_valid_iban_tr(&iban), "{iban}");
            let card = compact(card_number(&mut rng));
            assert!(is_valid_card_number(&card), "{card}");
        }
    }
}
