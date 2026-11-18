//! Adres ve kurum.

use super::Rng;
use super::data::{BRAND_WORDS, DISTRICTS, PERSON_PLACE_NAMES, PLACE_WORDS, PROVINCES, SECTORS};
use super::names::{full_name, surname};

fn province_and_district(rng: &mut Rng) -> (&'static str, &'static str, u8) {
    let (province, code) = *rng.pick(PROVINCES);
    let district = DISTRICTS
        .iter()
        .find(|(p, _)| *p == province)
        .map_or("Merkez", |(_, ds)| *rng.pick(ds));
    (province, district, code)
}

/// Mahalle düzeyinde ya da daha ayrıntılı adres; bitişik tek span:
/// `Gazi Mah. Lale Cad. No: 12 D: 3 Çankaya/Ankara`.
///
/// `person_named` ise cadde bir kişinin adını taşır ("Atatürk Caddesi");
/// etiketleme yönergesine göre bu ad adresin parçasıdır.
pub fn address(rng: &mut Rng, person_named: bool) -> String {
    let (province, district, code) = province_and_district(rng);
    let neighbourhood = format!(
        "{} {}",
        rng.pick(PLACE_WORDS),
        rng.pick(&["Mah.", "Mahallesi"])
    );
    let street = if person_named {
        format!(
            "{} {}",
            rng.pick(PERSON_PLACE_NAMES),
            rng.pick(&["Cad.", "Caddesi", "Bulvarı", "Sok."])
        )
    } else if rng.chance(25) {
        format!("{}. {}", rng.range(1, 2999), rng.pick(&["Sokak", "Sok."]))
    } else if rng.chance(5) {
        format!("{}. Cadde", rng.range(1, 40))
    } else {
        format!(
            "{} {}",
            rng.pick(PLACE_WORDS),
            rng.pick(&["Cad.", "Caddesi", "Sokak", "Sok."])
        )
    };
    let no = rng.range(1, 250);
    let flat = rng.range(1, 40);
    match rng.below(4) {
        0 => format!("{neighbourhood} {street} No: {no} D: {flat} {district}/{province}"),
        1 => format!(
            "{neighbourhood}, {street} No:{no}/{flat}, {code:02}{} {district}/{province}",
            rng.digits(3)
        ),
        2 => format!(
            "{street} No: {no} Kat: {} {district} / {province}",
            rng.range(1, 12)
        ),
        _ => format!("{neighbourhood} {street} No {no} Daire {flat} {district} {province}"),
    }
}

/// Kurum türü.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrgKind {
    /// İşyeri.
    Employer,
    /// Okul ya da üniversite.
    School,
    /// Hastane ya da sağlık kurumu.
    Hospital,
    /// Banka.
    Bank,
    /// Diğer: sigorta, belediye, spor kulübü.
    Other,
}

impl OrgKind {
    /// Tüm türler.
    pub const ALL: &'static [OrgKind] = &[
        OrgKind::Employer,
        OrgKind::School,
        OrgKind::Hospital,
        OrgKind::Bank,
        OrgKind::Other,
    ];

    /// Rastgele bir tür.
    pub fn any(rng: &mut Rng) -> Self {
        *rng.pick(Self::ALL)
    }
}

/// Kurum adı, yasal ekiyle: `Kaya Lojistik Ltd. Şti.`, `Kadıköy Devlet Hastanesi`.
///
/// `person_named` ise ad bir kişi adı taşır ("Ahmet Yılmaz İlkokulu");
/// yönergeye göre tek `ORGANIZATION` span'idir. Banka ve diğer türlerde
/// yok sayılır.
pub fn organization(rng: &mut Rng, kind: OrgKind, person_named: bool) -> String {
    let (province, district, _) = province_and_district(rng);
    let place = if district == "Merkez" {
        province
    } else {
        district
    };
    let brand = *rng.pick(BRAND_WORDS);
    match kind {
        OrgKind::Employer => {
            let owner = if person_named {
                full_name(rng, None)
            } else if rng.chance(50) {
                surname(rng).to_owned()
            } else {
                brand.to_owned()
            };
            let suffix = *rng.pick(&[
                "A.Ş.",
                "Ltd. Şti.",
                "San. ve Tic. A.Ş.",
                "San. Tic. Ltd. Şti.",
            ]);
            format!("{owner} {} {suffix}", rng.pick(SECTORS))
        }
        OrgKind::School => {
            let name = if person_named {
                if rng.chance(50) {
                    full_name(rng, None)
                } else {
                    (*rng.pick(PERSON_PLACE_NAMES)).to_owned()
                }
            } else if rng.chance(50) {
                place.to_owned()
            } else {
                brand.to_owned()
            };
            format!(
                "{name} {}",
                rng.pick(&[
                    "İlkokulu",
                    "Ortaokulu",
                    "Anadolu Lisesi",
                    "Fen Lisesi",
                    "Meslek Yüksekokulu",
                    "Koleji"
                ])
            )
        }
        OrgKind::Hospital => {
            if person_named {
                format!("{} Devlet Hastanesi", rng.pick(PERSON_PLACE_NAMES))
            } else {
                match rng.below(4) {
                    0 => format!("{place} Devlet Hastanesi"),
                    1 => format!("Özel {brand} Hastanesi"),
                    2 => format!("{province} Eğitim ve Araştırma Hastanesi"),
                    _ => format!("{brand} Tıp Merkezi"),
                }
            }
        }
        OrgKind::Bank => format!(
            "{brand} {}",
            rng.pick(&["Bank", "Bankası", "Katılım Bankası"])
        ),
        OrgKind::Other => match rng.below(3) {
            0 => format!("{brand} Sigorta A.Ş."),
            1 => format!("{place} Belediyesi"),
            _ => format!("{brand}spor Kulübü"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn person_named_places_carry_a_name() {
        for seed in 0..300 {
            let mut rng = Rng::new(seed);
            let a = address(&mut rng, true);
            assert!(PERSON_PLACE_NAMES.iter().any(|n| a.contains(n)), "{a}");
            let a = address(&mut rng, false);
            assert!(a.contains('/') || a.split(' ').count() > 5, "{a}");
            for &kind in OrgKind::ALL {
                let o = organization(&mut rng, kind, seed % 2 == 0);
                assert!(!o.is_empty() && o.trim() == o, "{o}");
            }
        }
    }

    #[test]
    fn districts_belong_to_listed_provinces() {
        for (p, ds) in DISTRICTS {
            assert!(PROVINCES.iter().any(|(q, _)| q == p), "{p}");
            assert!(!ds.is_empty());
        }
        assert_eq!(PROVINCES.len(), 81);
        for (i, (_, code)) in PROVINCES.iter().enumerate() {
            assert_eq!(usize::from(*code), i + 1);
        }
    }
}
