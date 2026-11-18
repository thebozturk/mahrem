//! Yer tutuculara değer yerleştirme.

use std::collections::HashMap;

use mahrem_core::Label;
use mahrem_core::fake::{
    Gender, OrgKind, Rng, address, age, ambiguous_first_name, email, first_name, full_name,
    generate, organization, surname, username,
};
use mahrem_core::to_lowercase_tr;
use mahrem_dataset::{Domain, HardCase};

use crate::doc::Seg;
use crate::harmony::fits;
use crate::spec::Entity;
use crate::template::{Piece, suffix_after};

/// Bir değeri ararken en çok deneme.
const TRIES: usize = 80;

/// Doldurmayı etkileyen belge bilgisi.
#[derive(Debug, Clone, Copy)]
pub struct FillContext {
    /// Metin türü.
    pub domain: Domain,
    /// Uygulanacak zor durum.
    pub hard_case: Option<HardCase>,
}

#[derive(Default)]
struct SlotContext {
    suffixes: Vec<String>,
    gender: Option<Gender>,
    honorific: bool,
}

const MALE_BEFORE: &[&str] = &[
    "bay",
    "oğlum",
    "babam",
    "eşim bay",
    "abim",
    "dedem",
    "amcam",
    "dayım",
];
const FEMALE_BEFORE: &[&str] = &[
    "bayan", "kızım", "annem", "ablam", "ninem", "teyzem", "halam",
];

fn context(pieces: &[Piece]) -> HashMap<&str, SlotContext> {
    let mut ctx: HashMap<&str, SlotContext> = HashMap::new();
    for (i, piece) in pieces.iter().enumerate() {
        let Piece::Slot(key) = piece else { continue };
        let c = ctx.entry(key.as_str()).or_default();
        c.suffixes.push(suffix_after(pieces.get(i + 1)));
        if let Some(Piece::Text(after)) = pieces.get(i + 1) {
            let a = to_lowercase_tr(after);
            let a = a.trim_start();
            if a.starts_with("bey") {
                c.gender = Some(Gender::Male);
                c.honorific = true;
            } else if a.starts_with("hanım") {
                c.gender = Some(Gender::Female);
                c.honorific = true;
            }
        }
        if let Some(Piece::Text(before)) = i.checked_sub(1).and_then(|j| pieces.get(j)) {
            let b = to_lowercase_tr(before);
            let last = b
                .trim_end()
                .rsplit(char::is_whitespace)
                .next()
                .unwrap_or_default();
            if MALE_BEFORE.contains(&last) {
                c.gender.get_or_insert(Gender::Male);
            } else if FEMALE_BEFORE.contains(&last) {
                c.gender.get_or_insert(Gender::Female);
            }
        }
    }
    ctx
}

fn candidate(
    e: &Entity,
    slot: &SlotContext,
    fc: FillContext,
    first_of_kind: bool,
    first_person: Option<&str>,
    rng: &mut Rng,
) -> String {
    match e.label {
        Label::Person => {
            if fc.hard_case == Some(HardCase::AmbiguousName) && first_of_kind {
                ambiguous_first_name(rng).to_owned()
            } else if fc.domain == Domain::Log && rng.chance(50) {
                username(rng, None)
            } else if e.child {
                if rng.chance(80) {
                    first_name(rng, slot.gender).to_owned()
                } else {
                    full_name(rng, slot.gender)
                }
            } else if slot.honorific {
                match rng.below(3) {
                    0 => surname(rng).to_owned(),
                    1 => first_name(rng, slot.gender).to_owned(),
                    _ => full_name(rng, slot.gender),
                }
            } else {
                match rng.below(10) {
                    0 => first_name(rng, slot.gender).to_owned(),
                    1 => surname(rng).to_owned(),
                    _ => full_name(rng, slot.gender),
                }
            }
        }
        Label::Age => age(rng, e.child),
        Label::Email => {
            let use_name = rng.chance(50);
            email(rng, first_person.filter(|_| use_name))
        }
        Label::Address => address(
            rng,
            fc.hard_case == Some(HardCase::NameInPlace) && first_of_kind,
        ),
        Label::Organization => {
            let kind = OrgKind::any(rng);
            organization(
                rng,
                kind,
                fc.hard_case == Some(HardCase::NameInPlace) && first_of_kind,
            )
        }
        _ => e.preset.clone().unwrap_or_else(|| generate(e.label, rng)),
    }
}

/// Yer tutucuları değerlerle doldurur. Aynı yer tutucunun her geçişi aynı
/// değeri alır; değer o yer tutucuya eklenen her eke uyar ve metnin başka
/// bir yerinde geçmez.
///
/// # Errors
///
/// Eklere uyan değer bulunamazsa `ek-uyumsuz`.
pub fn fill(
    entities: &[Entity],
    pieces: &[Piece],
    fc: FillContext,
    rng: &mut Rng,
) -> Result<Vec<Seg>, &'static str> {
    let ctx = context(pieces);
    let plain: String = pieces
        .iter()
        .filter_map(|p| match p {
            Piece::Text(t) => Some(to_lowercase_tr(t)),
            Piece::Slot(_) => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    let mut values: HashMap<&str, String> = HashMap::new();
    let mut first_person: Option<String> = None;
    let empty = SlotContext::default();
    for (i, e) in entities.iter().enumerate() {
        let slot = ctx.get(e.key.as_str()).unwrap_or(&empty);
        // NameInPlace adres ya da kurumdan ilkine; AmbiguousName ilk kişiye.
        let first_of_kind = match e.label {
            Label::Address | Label::Organization => !entities[..i]
                .iter()
                .any(|x| matches!(x.label, Label::Address | Label::Organization)),
            l => !entities[..i].iter().any(|x| x.label == l),
        };
        let tries = if e.preset.is_some() { 1 } else { TRIES };
        let mut found = None;
        for _ in 0..tries {
            let v = candidate(e, slot, fc, first_of_kind, first_person.as_deref(), rng);
            let lower = to_lowercase_tr(&v);
            let clashes = (v.chars().count() >= 3 && plain.contains(&lower))
                || values.values().any(|x| to_lowercase_tr(x) == lower);
            if !clashes && slot.suffixes.iter().all(|s| fits(&v, s)) {
                found = Some(v);
                break;
            }
        }
        let v = found.ok_or("ek-uyumsuz")?;
        if e.label == Label::Person && first_person.is_none() && v.contains(' ') {
            first_person = Some(v.clone());
        }
        values.insert(e.key.as_str(), v);
    }
    let by_key: HashMap<&str, &Entity> = entities.iter().map(|e| (e.key.as_str(), e)).collect();
    Ok(pieces
        .iter()
        .map(|p| match p {
            Piece::Text(t) => Seg::plain(t.clone()),
            Piece::Slot(k) => {
                let e = by_key[k.as_str()];
                Seg {
                    text: values[k.as_str()].clone(),
                    span: Some((e.label, e.child)),
                }
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::parse;

    fn ent(label: Label, key: &str, child: bool, preset: Option<&str>) -> Entity {
        Entity {
            label,
            key: key.into(),
            child,
            preset: preset.map(Into::into),
        }
    }

    const FC: FillContext = FillContext {
        domain: Domain::Chat,
        hard_case: None,
    };

    #[test]
    fn fills_with_harmony_and_gender() {
        let es = vec![
            ent(Label::Person, "KİŞİ_1", false, None),
            ent(Label::Person, "KİŞİ_2", true, None),
            ent(Label::Health, "SAĞLIK_1", false, Some("astım")),
        ];
        let keys: Vec<String> = es.iter().map(|e| e.key.clone()).collect();
        let pieces = parse(
            "Ben {KİŞİ_1} Hanım. Kızım {KİŞİ_2}'nin {SAĞLIK_1}'ı var, {KİŞİ_1}'e ulaşın.",
            &keys,
        )
        .unwrap();
        for seed in 0..200 {
            let segs = fill(&es, &pieces, FC, &mut Rng::new(seed)).unwrap();
            let values: Vec<&Seg> = segs.iter().filter(|s| s.span.is_some()).collect();
            assert_eq!(values.len(), 4);
            assert_eq!(values[0].text, values[3].text, "aynı yer tutucu aynı değer");
            assert!(fits(&values[1].text, "nin"), "{}", values[1].text);
            assert!(fits(&values[0].text, "e"), "{}", values[0].text);
            assert_eq!(values[2].text, "astım");
            assert_eq!(values[1].span, Some((Label::Person, true)));
        }
    }

    #[test]
    fn preset_that_does_not_fit_is_rejected() {
        let es = vec![ent(Label::Health, "SAĞLIK_1", false, Some("astım"))];
        let pieces = parse("{SAĞLIK_1}'e yakalandım", &["SAĞLIK_1".to_owned()]).unwrap();
        assert_eq!(fill(&es, &pieces, FC, &mut Rng::new(1)), Err("ek-uyumsuz"));
    }
}
