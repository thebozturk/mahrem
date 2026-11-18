//! KVKK m.6 özel nitelikli verinin değer sözcükleri.
//!
//! Etiketleme yönergesine göre span, bilgiyi açığa çıkaran değer sözcüğüdür;
//! yüklem ve ek dışarıda kalır: `[diyabet] hastasıyım`. Bu listeler yalnızca
//! o sözcükleri verir. Siyasi parti, dernek ve sendika adları uydurmadır.

use super::Rng;
use super::data::BRAND_WORDS;
use crate::Label;

const HEALTH: &[&str] = &[
    "diyabet",
    "astım",
    "hipertansiyon",
    "migren",
    "epilepsi",
    "depresyon",
    "anksiyete bozukluğu",
    "panik atak",
    "lösemi",
    "meme kanseri",
    "çölyak",
    "MS",
    "Crohn hastalığı",
    "böbrek yetmezliği",
    "kalp yetmezliği",
    "HIV",
    "hepatit B",
    "otizm",
    "DEHB",
    "disleksi",
    "skolyoz",
    "bel fıtığı",
    "hamile",
    "insülin",
    "antidepresan",
    "kemoterapi",
    "diyaliz",
    "grip",
    "alerji",
    "gastrit",
    "reflü",
    "işitme kaybı",
    "görme engeli",
    "Alzheimer",
    "Parkinson",
    "şizofreni",
    "bipolar bozukluk",
    "tiroit",
    "romatizma",
    "obezite",
];

const BELIEF: &[&str] = &[
    "Müslüman",
    "Sünni",
    "Alevi",
    "Şii",
    "Hristiyan",
    "Katolik",
    "Ortodoks",
    "Protestan",
    "Yahudi",
    "Ezidi",
    "Budist",
    "ateist",
    "agnostik",
    "deist",
    "Bahai",
    "oruç",
    "namaz",
];

const ETHNIC_ORIGIN: &[&str] = &[
    "Kürt",
    "Laz",
    "Çerkez",
    "Ermeni",
    "Rum",
    "Roman",
    "Arap",
    "Gürcü",
    "Boşnak",
    "Arnavut",
    "Zaza",
    "Abhaz",
    "Pomak",
    "Türkmen",
    "Yörük",
    "Süryani",
    "Afro-Türk",
];

const POLITICAL: &[&str] = &[
    "sosyalist",
    "muhafazakâr",
    "milliyetçi",
    "liberal",
    "sosyal demokrat",
    "komünist",
    "anarşist",
    "Mavi Ufuk Partisi",
    "Yeşil Vadi Partisi",
    "Kardelen Partisi",
    "Pusula Partisi",
    "Çınar Partisi",
];

const ATTIRE: &[&str] = &[
    "başörtüsü",
    "türban",
    "çarşaf",
    "peçe",
    "kipa",
    "sarık",
    "nikap",
    "burka",
];

const SEX_LIFE: &[&str] = &[
    "eşcinsel",
    "gey",
    "lezbiyen",
    "biseksüel",
    "aseksüel",
    "panseksüel",
];

const CRIMINAL: &[&str] = &[
    "hapis",
    "sabıka",
    "adli kontrol",
    "ev hapsi",
    "denetimli serbestlik",
    "tutukluluk",
    "hırsızlık",
    "dolandırıcılık",
    "kasten yaralama",
    "hapis cezası",
    "adli sicil kaydı",
];

const GENETIC: &[&str] = &[
    "BRCA1 mutasyonu",
    "BRCA2 mutasyonu",
    "talasemi taşıyıcılığı",
    "kistik fibrozis taşıyıcılığı",
    "SMA taşıyıcılığı",
    "Faktör V Leiden mutasyonu",
    "MTHFR mutasyonu",
    "APOE4 geni",
];

fn hex(rng: &mut Rng, n: usize) -> String {
    (0..n)
        .map(|_| char::from(*rng.pick(b"0123456789ABCDEF")))
        .collect()
}

/// Özel nitelikli bir etiket için değer sözcüğü; diğer etiketlerde `None`.
pub fn special_category(label: Label, rng: &mut Rng) -> Option<String> {
    let pick = |rng: &mut Rng, list: &[&str]| (*rng.pick(list)).to_owned();
    Some(match label {
        Label::Health => pick(rng, HEALTH),
        Label::Belief => pick(rng, BELIEF),
        Label::EthnicOrigin => pick(rng, ETHNIC_ORIGIN),
        Label::PoliticalOpinion => pick(rng, POLITICAL),
        Label::Attire => pick(rng, ATTIRE),
        Label::SexLife => pick(rng, SEX_LIFE),
        Label::CriminalRecord => pick(rng, CRIMINAL),
        Label::Genetic => pick(rng, GENETIC),
        Label::Membership => match rng.below(5) {
            0 => pick(rng, &["sendika", "dernek", "vakıf"]),
            1 => format!("{}-İş", rng.pick(BRAND_WORDS)),
            2 => format!("{}-Sen", rng.pick(BRAND_WORDS)),
            3 => format!("{} Derneği", rng.pick(BRAND_WORDS)),
            _ => format!("{} Vakfı", rng.pick(BRAND_WORDS)),
        },
        // Biyometrik verinin kendisini temsil eden şablon kodu.
        Label::Biometric => match rng.below(3) {
            0 => format!("FP-{}", hex(rng, 24)),
            1 => format!("IRIS-{}", hex(rng, 16)),
            _ => hex(rng, 64),
        },
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_special_categories_have_values() {
        let mut rng = Rng::new(3);
        for &label in Label::ALL {
            assert_eq!(
                special_category(label, &mut rng).is_some(),
                label.is_special_category(),
                "{label}"
            );
        }
    }
}
