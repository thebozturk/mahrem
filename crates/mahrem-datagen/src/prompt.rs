//! LLM isteği.

use mahrem_core::Label;
use mahrem_dataset::Domain;
use mahrem_llm::Message;

use crate::spec::{DocSpec, Entity, SYSTEM_PROMPT, Scenarios};

/// Prompt kurgusunun sürümü; değişince şablon özeti değişir.
pub const PROMPT_VERSION: &str = "1";

fn describe(e: &Entity, spec: &DocSpec) -> String {
    use Label as L;
    let first_child = spec
        .entities
        .iter()
        .find(|x| x.label == L::Person && x.child)
        .map(|x| x.key.as_str());
    let first_person = spec
        .entities
        .iter()
        .find(|x| x.label == L::Person)
        .map(|x| x.key.as_str());
    let base = match e.label {
        L::Person if e.child => "bir çocuğun adı (örneğin yazanın kızı, oğlu ya da öğrencisi)".into(),
        L::Person if Some(e.key.as_str()) == first_person => {
            if spec.domain == Domain::Log {
                "bir kullanıcının adı ya da kullanıcı adı".into()
            } else {
                "metni yazan ya da metnin sözünü ettiği asıl kişinin adı".into()
            }
        }
        L::Person => "başka bir kişinin adı".into(),
        L::Age => match (e.child, first_child, first_person) {
            (true, Some(c), _) => format!("{{{c}}}'nin yaşı, yalnızca sayı; \"yaşında\" sözcüğü dışarıda"),
            (_, _, Some(p)) => format!("{{{p}}}'nin yaşı, yalnızca sayı; \"yaşında\" sözcüğü dışarıda"),
            _ => "bir yetişkinin yaşı, yalnızca sayı; \"yaşında\" sözcüğü dışarıda".into(),
        },
        L::Date => "bir kişinin doğum tarihi ya da sağlığıyla ilgili bir tarih (ameliyat, taburcu)".into(),
        L::Tckn => "T.C. kimlik numarası".into(),
        L::Ykn => "yabancı kimlik numarası".into(),
        L::Vkn => "vergi kimlik numarası".into(),
        L::Iban => "IBAN".into(),
        L::CardNumber => "kredi kartı numarasının tamamı".into(),
        L::Phone => "telefon numarası".into(),
        L::Email => "e-posta adresi".into(),
        L::Passport => "pasaport numarası".into(),
        L::DriverLicense => "sürücü belgesi numarası".into(),
        L::SgkNumber => "SGK sicil numarası".into(),
        L::Mersis => "bir şirketin MERSİS numarası".into(),
        L::AccountId => "müşteri, abone, poliçe ya da hasta numarası (sipariş ya da kargo numarası değil)".into(),
        L::LicensePlate => "araç plakası".into(),
        L::SerialNumber => "cihaz seri numarası ya da IMEI".into(),
        L::IpAddress => "IP adresi".into(),
        L::MacAddress => "MAC adresi".into(),
        L::Secret => "parola, API anahtarı ya da tek kullanımlık doğrulama kodu".into(),
        L::Address => "açık adres (mahalle, cadde, numara, ilçe ve il)".into(),
        L::Organization => {
            "kişinin çalıştığı işyeri, okulu, hastanesi ya da bankası; mesajın gönderildiği kurum değil".into()
        }
        L::EthnicOrigin => special("bir kişinin etnik kökeni", e),
        L::PoliticalOpinion => special("bir kişinin siyasi görüşü ya da desteklediği parti", e),
        L::Belief => special("bir kişinin dini, mezhebi, inancı ya da ibadeti", e),
        L::Attire => special("bir kişinin dini ya da kültürel kıyafeti", e),
        L::Membership => special("bir kişinin üye olduğu dernek, vakıf ya da sendika", e),
        L::Health => special("bir kişinin sağlık bilgisi", e),
        L::SexLife => special("bir kişinin cinsel yönelimi", e),
        L::CriminalRecord => special("bir kişinin ceza kaydı ya da yargılandığı suç", e),
        L::Biometric => special("bir kişinin biyometrik şablon kodu", e),
        L::Genetic => special("bir kişinin genetik test sonucu", e),
    };
    format!("- {{{}}}: {base}", e.key)
}

fn special(what: &str, e: &Entity) -> String {
    let value = e.preset.as_deref().unwrap_or_default();
    format!(
        "{what}. Yerine \"{value}\" gelecek; bu sözcüğü yazma, yer tutucuyu yaz. \
         Yalnızca bu değer yer tutucudur, \"hastasıyım\", \"üyesiyim\" gibi sözcükler dışarıda kalır"
    )
}

fn length_hint(domain: Domain) -> &'static str {
    match domain {
        Domain::Log => "4-8 log satırı yaz.",
        Domain::Form => "Alan adı ve değer satırları ile kısa bir açıklama yaz.",
        Domain::Document => "Bir paragraf yaz.",
        _ => "2-6 cümle yaz.",
    }
}

/// Belge planından LLM mesajları.
#[must_use]
pub fn messages(spec: &DocSpec, scenarios: &Scenarios) -> Vec<Message> {
    let domain = scenarios.domains.get(&spec.domain);
    let description = domain.map_or("metin", |d| d.description.as_str());
    let scenario = domain
        .and_then(|d| d.scenarios.get(spec.scenario))
        .map_or("serbest konu", String::as_str);
    let mut user = format!("Metin türü: {description}.\nKonu: {scenario}.\n");
    if spec.entities.is_empty() {
        user.push_str("Bu metinde hiçbir kişisel veri olmasın; yer tutucu kullanma.\n");
    } else {
        user.push_str("Yer tutucular:\n");
        for e in &spec.entities {
            user.push_str(&describe(e, spec));
            user.push('\n');
        }
        user.push_str("Her yer tutucuyu bir kez kullan.\n");
    }
    user.push_str(length_hint(spec.domain));
    vec![Message::system(SYSTEM_PROMPT.trim()), Message::user(user)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::plan;

    #[test]
    fn every_placeholder_is_described() {
        let s = Scenarios::embedded();
        for seed in 0..300 {
            let p = plan(seed, &s);
            let msgs = messages(&p, &s);
            let user = &msgs[1].content;
            for e in &p.entities {
                assert!(user.contains(&format!("{{{}}}", e.key)), "{user}");
                if let Some(v) = &e.preset {
                    assert!(user.contains(v.as_str()), "{user}");
                }
            }
        }
    }
}
