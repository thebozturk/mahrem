//! Kişisel veri etiketleri.
//!
//! Her etiketin iki adı vardır:
//!
//! - **Kimlik** ([`Label::id`]): veri dosyalarında, benchmark çıktısında ve
//!   API'de kullanılan kalıcı ASCII ad (`PERSON`). Değişmez; bir kimliği
//!   değiştirmek yayınlanmış veri setlerini bozar.
//! - **Placeholder adı** ([`Label::placeholder_name`]): LLM'e giden metindeki
//!   Türkçe varsayılan ad (`<KİŞİ_1>`). Politikayla değiştirilebilir.
//!
//! Gerekçe: `docs/decisions/0003-etiket-kimlikleri.md`.

use std::fmt;
use std::str::FromStr;

macro_rules! labels {
    ($($variant:ident => $id:literal, $placeholder:literal, $doc:literal;)+) => {
        /// Kişisel veri etiketi.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum Label {
            $(
                #[doc = $doc]
                $variant,
            )+
        }

        impl Label {
            /// Tüm etiketler, tanım sırasıyla.
            pub const ALL: &'static [Label] = &[$(Label::$variant),+];

            /// Kalıcı ASCII kimlik (`PERSON`).
            #[must_use]
            pub const fn id(self) -> &'static str {
                match self {
                    $(Label::$variant => $id,)+
                }
            }

            /// Varsayılan Türkçe placeholder adı (`KİŞİ`).
            #[must_use]
            pub const fn placeholder_name(self) -> &'static str {
                match self {
                    $(Label::$variant => $placeholder,)+
                }
            }
        }
    };
}

labels! {
    Tckn => "TCKN", "TCKN", "T.C. kimlik numarası.";
    Ykn => "YKN", "YKN", "Yabancı kimlik numarası.";
    Vkn => "VKN", "VKN", "Vergi kimlik numarası.";
    Iban => "IBAN", "IBAN", "IBAN.";
    CardNumber => "CARD_NUMBER", "KART", "Ödeme kartı numarası.";
    Phone => "PHONE", "TELEFON", "Cep ya da sabit telefon numarası.";
    Email => "EMAIL", "E_POSTA", "E-posta adresi.";
    Passport => "PASSPORT", "PASAPORT", "Pasaport numarası.";
    DriverLicense => "DRIVER_LICENSE", "EHLİYET", "Sürücü belgesi numarası.";
    SgkNumber => "SGK_NUMBER", "SGK_NO", "SGK sicil numarası.";
    Mersis => "MERSIS", "MERSİS", "MERSİS numarası.";
    LicensePlate => "LICENSE_PLATE", "PLAKA", "Araç plakası.";
    SerialNumber => "SERIAL_NUMBER", "SERİ_NO", "Cihaz seri numarası.";
    IpAddress => "IP_ADDRESS", "IP", "IP adresi.";
    MacAddress => "MAC_ADDRESS", "MAC", "MAC adresi.";
    Date => "DATE", "TARİH", "Bir kişiye bağlı tarih (doğum tarihi gibi).";
    Secret => "SECRET", "GİZLİ", "API anahtarı, JWT, özel anahtar ya da parola.";
    Person => "PERSON", "KİŞİ", "Kişi adı.";
    Address => "ADDRESS", "ADRES", "Adres; bitişik adres tek varlıktır.";
    Organization => "ORGANIZATION", "KURUM", "Kurum: işveren, okul, hastane.";
    Age => "AGE", "YAŞ", "Yaş.";
    EthnicOrigin => "ETHNIC_ORIGIN", "ETNİK_KÖKEN", "KVKK m.6: ırk ve etnik köken.";
    PoliticalOpinion => "POLITICAL_OPINION", "SİYASİ_GÖRÜŞ", "KVKK m.6: siyasi düşünce.";
    Belief => "BELIEF", "İNANÇ", "KVKK m.6: felsefi inanç, din, mezhep ve diğer inançlar.";
    Attire => "ATTIRE", "KIYAFET", "KVKK m.6: kılık ve kıyafet.";
    Membership => "MEMBERSHIP", "ÜYELİK", "KVKK m.6: dernek, vakıf ya da sendika üyeliği.";
    Health => "HEALTH", "SAĞLIK", "KVKK m.6: sağlık.";
    SexLife => "SEX_LIFE", "CİNSEL_HAYAT", "KVKK m.6: cinsel hayat.";
    CriminalRecord => "CRIMINAL_RECORD", "CEZA_KAYDI", "KVKK m.6: ceza mahkûmiyeti ve güvenlik tedbirleri.";
    Biometric => "BIOMETRIC", "BİYOMETRİK", "KVKK m.6: biyometrik veri.";
    Genetic => "GENETIC", "GENETİK", "KVKK m.6: genetik veri.";
}

impl Label {
    /// KVKK m.6/1 kapsamındaki özel nitelikli veri mi?
    #[must_use]
    pub const fn is_special_category(self) -> bool {
        matches!(
            self,
            Label::EthnicOrigin
                | Label::PoliticalOpinion
                | Label::Belief
                | Label::Attire
                | Label::Membership
                | Label::Health
                | Label::SexLife
                | Label::CriminalRecord
                | Label::Biometric
                | Label::Genetic
        )
    }

    /// Yüksek riskli grupta mı? Tanım PRD'deki ölçüm hedefiyle aynıdır:
    /// TCKN, YKN, VKN, IBAN, kart ve KVKK m.6.
    #[must_use]
    pub const fn is_high_risk(self) -> bool {
        matches!(
            self,
            Label::Tckn | Label::Ykn | Label::Vkn | Label::Iban | Label::CardNumber
        ) || self.is_special_category()
    }

    /// Bu etiket "çocuğa ait" diye işaretlenebilir mi? Yalnızca kişi adı ve yaş.
    #[must_use]
    pub const fn allows_child_flag(self) -> bool {
        matches!(self, Label::Person | Label::Age)
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id())
    }
}

/// Bilinmeyen etiket kimliği.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownLabel(pub String);

impl fmt::Display for UnknownLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bilinmeyen etiket: {:?}", self.0)
    }
}

impl std::error::Error for UnknownLabel {}

impl FromStr for Label {
    type Err = UnknownLabel;

    /// Kimlikten etiket üretir. Büyük/küçük harf duyarlıdır.
    ///
    /// ```
    /// use mahrem_core::Label;
    /// assert_eq!("PERSON".parse::<Label>(), Ok(Label::Person));
    /// assert!("person".parse::<Label>().is_err());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Label::ALL
            .iter()
            .copied()
            .find(|l| l.id() == s)
            .ok_or_else(|| UnknownLabel(s.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn ids_are_unique_ascii_upper_snake() {
        let mut seen = HashSet::new();
        for l in Label::ALL {
            let id = l.id();
            assert!(seen.insert(id), "tekrar eden kimlik: {id}");
            assert!(
                id.bytes()
                    .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_'),
                "{id}"
            );
        }
    }

    #[test]
    fn placeholder_names_are_unique() {
        let mut seen = HashSet::new();
        for l in Label::ALL {
            assert!(seen.insert(l.placeholder_name()), "{l}");
        }
    }

    #[test]
    fn round_trip() {
        for &l in Label::ALL {
            assert_eq!(l.id().parse::<Label>(), Ok(l));
        }
    }

    #[test]
    fn groups() {
        assert_eq!(
            Label::ALL
                .iter()
                .filter(|l| l.is_special_category())
                .count(),
            10
        );
        assert!(Label::Tckn.is_high_risk());
        assert!(Label::Health.is_high_risk());
        assert!(!Label::Person.is_high_risk());
        assert!(!Label::Secret.is_high_risk());
        assert!(Label::Person.allows_child_flag());
        assert!(!Label::Phone.allows_child_flag());
    }
}
