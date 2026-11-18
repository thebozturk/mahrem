//! Belge planı: metin türü, senaryo, varlıklar ve zor durum tohumdan belirlenir.
//!
//! Oranlar `docs/gold-set/composition-v0.json`'daki aralıkların ortasını
//! hedefler; seyrek etiketler de en az span sayısına ulaşacak ağırlıkta
//! seçilir. Planın hedefi tutturduğu testte denetlenir.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use mahrem_core::Label;
use mahrem_core::fake::{Rng, special_category};
use mahrem_dataset::{Domain, HardCase};
use serde::Deserialize;
use sha2::{Digest, Sha256};

/// Sistem prompt'u.
pub const SYSTEM_PROMPT: &str = include_str!("../prompts/system.txt");
/// Senaryolar.
pub const SCENARIOS: &str = include_str!("../prompts/scenarios.toml");

/// Prompt'ların ve senaryoların özeti; köken kaydına yazılır.
#[must_use]
pub fn template_hash() -> String {
    let mut h = Sha256::new();
    h.update(SYSTEM_PROMPT.as_bytes());
    h.update([0]);
    h.update(SCENARIOS.as_bytes());
    h.update([0]);
    h.update(crate::prompt::PROMPT_VERSION.as_bytes());
    h.finalize().iter().fold(String::new(), |mut out, b| {
        let _ = write!(out, "{b:02x}");
        out
    })
}

/// Bir metin türünün senaryoları.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DomainScenarios {
    /// Metin türünün LLM'e anlatımı.
    pub description: String,
    /// Senaryolar.
    pub scenarios: Vec<String>,
}

/// Tüm senaryolar.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenarios {
    /// Metin türü → senaryolar.
    pub domains: BTreeMap<Domain, DomainScenarios>,
}

impl Scenarios {
    /// Gömülü senaryoları okur.
    ///
    /// # Panics
    ///
    /// Gömülü dosya bozuksa; testte denetlenir.
    #[must_use]
    pub fn embedded() -> Self {
        toml::from_str(SCENARIOS).expect("prompts/scenarios.toml okunamadı")
    }
}

/// Belgedeki bir varlık.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    /// Etiket.
    pub label: Label,
    /// Yer tutucu adı: `KİŞİ_1`.
    pub key: String,
    /// Çocuğa ait mi (yalnızca `PERSON` ve `AGE`).
    pub child: bool,
    /// Önceden seçilmiş değer: özel nitelikli veride anlam metne yön verir,
    /// bu yüzden LLM'e söylenir.
    pub preset: Option<String>,
}

/// Bir belgenin planı.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocSpec {
    /// Metin türü.
    pub domain: Domain,
    /// Senaryo sırası.
    pub scenario: usize,
    /// Varlıklar.
    pub entities: Vec<Entity>,
    /// Uygulanacak zor durum.
    pub hard_case: Option<HardCase>,
    /// Bölme grubu: metin türü ve senaryo.
    pub group: String,
}

/// Metin türlerinin ağırlıkları (binde), kompozisyon aralıklarının ortası.
const DOMAINS: &[(Domain, u64)] = &[
    (Domain::CustomerSupport, 300),
    (Domain::Chat, 200),
    (Domain::Email, 150),
    (Domain::Form, 100),
    (Domain::Document, 100),
    (Domain::SocialMedia, 100),
    (Domain::Log, 65),
    (Domain::Other, 25),
];

/// Belge başına varlık sayısı kovaları: (en az, en çok, ağırlık).
const BUCKETS: &[(u64, u64, u64)] = &[(0, 0, 20), (1, 1, 20), (2, 3, 30), (4, 5, 20), (6, 8, 10)];

/// Zor durum içeren belge oranı (yüzde, varlık içeren belgeler arasında).
const HARD_CASE_PERCENT: u64 = 25;

fn weight(label: Label, domain: Domain) -> u64 {
    use Label as L;
    if domain == Domain::Log {
        return match label {
            L::IpAddress => 8,
            L::Email | L::Secret | L::AccountId => 5,
            L::MacAddress | L::SerialNumber | L::Person | L::Phone => 3,
            L::Tckn | L::Iban | L::CardNumber => 2,
            l if l.is_special_category() => 0,
            L::Age | L::Address | L::Organization => 0,
            _ => 1,
        };
    }
    match label {
        L::Person => 14,
        L::Phone => 6,
        L::Email | L::Address | L::Organization => 5,
        L::Date | L::Age | L::Tckn => 4,
        L::Iban | L::AccountId => 3,
        L::CardNumber => 2,
        l if l.is_special_category() => 2,
        _ => 1,
    }
}

fn weighted<T: Copy>(rng: &mut Rng, items: &[(T, u64)]) -> T {
    let total: u64 = items.iter().map(|(_, w)| w).sum();
    let mut x = rng.below(total);
    for &(item, w) in items {
        if x < w {
            return item;
        }
        x -= w;
    }
    unreachable!("ağırlıklar toplamı aşılmaz")
}

/// Zor durumun gerektirdiği etiketlerden biri; `None` ise her belge olur.
fn required_labels(case: HardCase) -> Option<&'static [Label]> {
    use Label as L;
    match case {
        HardCase::NoDiacritics | HardCase::AllCaps | HardCase::AllLowercase => None,
        HardCase::Typo => Some(&[L::Person, L::Address, L::Organization]),
        HardCase::AmbiguousName => Some(&[L::Person]),
        HardCase::NameInPlace => Some(&[L::Address, L::Organization]),
        HardCase::SpelledNumbers => Some(&[L::Phone, L::Age]),
        HardCase::SpacedNumbers | HardCase::SplitLines => {
            Some(&[L::Phone, L::Tckn, L::Iban, L::CardNumber])
        }
        HardCase::ObfuscatedEmail => Some(&[L::Email]),
    }
}

/// Tohumdan belge planı.
#[must_use]
pub fn plan(seed: u64, scenarios: &Scenarios) -> DocSpec {
    let mut rng = Rng::new(seed);
    let domain = weighted(&mut rng, DOMAINS);
    let n_scenarios = scenarios
        .domains
        .get(&domain)
        .map_or(1, |d| d.scenarios.len());
    let scenario = usize::try_from(rng.below(n_scenarios as u64)).unwrap_or(0);

    let bucket = weighted(
        &mut rng,
        &BUCKETS
            .iter()
            .map(|&(lo, hi, w)| ((lo, hi), w))
            .collect::<Vec<_>>(),
    );
    let count = rng.range(bucket.0, bucket.1);
    let label_weights: Vec<(Label, u64)> = Label::ALL
        .iter()
        .map(|&l| (l, weight(l, domain)))
        .filter(|&(_, w)| w > 0)
        .collect();
    let mut labels: Vec<Label> = (0..count)
        .map(|_| weighted(&mut rng, &label_weights))
        .collect();

    let hard_case = if labels.is_empty() || !rng.chance(HARD_CASE_PERCENT) {
        None
    } else {
        let case = *rng.pick(HardCase::ALL);
        if let Some(required) = required_labels(case)
            && !labels.iter().any(|l| required.contains(l))
        {
            let i = usize::try_from(rng.below(labels.len() as u64)).unwrap_or(0);
            labels[i] = *rng.pick(required);
        }
        Some(case)
    };

    // Kişi adı belgenin başında, yaş kişiden sonra gelsin: prompt'ta
    // "yaş, şu kişinin yaşı" diyebilmek için.
    labels.sort_by_key(|l| match l {
        Label::Person => 0,
        Label::Age => 2,
        _ => 1,
    });
    let mut per_label: BTreeMap<Label, usize> = BTreeMap::new();
    let mut entities = Vec::with_capacity(labels.len());
    let mut has_child_person = false;
    for label in labels {
        let n = per_label.entry(label).or_default();
        *n += 1;
        let key = format!("{}_{}", label.placeholder_name(), n);
        let child = match label {
            // İlk kişi metnin yazarı ya da asıl kişisidir; sonrakiler çocuk olabilir.
            Label::Person => {
                let child = *n > 1 && domain != Domain::Log && rng.chance(40)
                    || *n == 1 && domain != Domain::Log && rng.chance(8);
                has_child_person |= child;
                child
            }
            Label::Age => has_child_person,
            _ => false,
        };
        let preset = special_category(label, &mut rng);
        entities.push(Entity {
            label,
            key,
            child,
            preset,
        });
    }
    DocSpec {
        domain,
        scenario,
        entities,
        hard_case,
        group: format!("{}-{scenario:02}", domain_key(domain)),
    }
}

/// Metin türünün kayıttaki adı (`customer_support`).
#[must_use]
pub fn domain_key(domain: Domain) -> String {
    serde_json::to_value(domain)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mahrem_dataset::{Composition, GoldRecord, GoldSpan, Meta, Origin, Source, Stats};

    #[test]
    fn scenarios_cover_every_domain() {
        let s = Scenarios::embedded();
        for &(d, _) in DOMAINS {
            assert!(
                s.domains.get(&d).is_some_and(|x| !x.scenarios.is_empty()),
                "{d:?}"
            );
        }
        assert_eq!(template_hash().len(), 64);
    }

    #[test]
    fn plans_are_deterministic_and_consistent() {
        let s = Scenarios::embedded();
        for seed in 0..2000 {
            let p = plan(seed, &s);
            assert_eq!(p, plan(seed, &s));
            for e in &p.entities {
                assert!(!e.child || e.label.allows_child_flag(), "{e:?}");
                assert_eq!(e.preset.is_some(), e.label.is_special_category());
            }
            if let Some(case) = p.hard_case
                && let Some(req) = required_labels(case)
            {
                assert!(p.entities.iter().any(|e| req.contains(&e.label)), "{p:?}");
            }
            let keys: std::collections::HashSet<_> = p.entities.iter().map(|e| &e.key).collect();
            assert_eq!(keys.len(), p.entities.len());
        }
    }

    /// Planlanan dağılım (her şey kabul edilse) bir bölümün hedeflerini
    /// tutturmalı: 650 plan ≈ ret ve bölmeden sonra 500 belge.
    #[test]
    fn planned_distribution_meets_composition() {
        let s = Scenarios::embedded();
        let records: Vec<GoldRecord> = (0..650u64)
            .map(|seed| {
                let p = plan(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15), &s);
                let mut text = String::new();
                let spans = p
                    .entities
                    .iter()
                    .map(|e| {
                        let start = text.chars().count();
                        text.push_str("x ");
                        GoldSpan {
                            start,
                            end: start + 1,
                            label: e.label,
                            text: "x".into(),
                            uncertain: false,
                            child: e.child,
                        }
                    })
                    .collect();
                GoldRecord {
                    id: format!("p{seed}"),
                    text,
                    spans,
                    meta: Meta {
                        schema_version: 1,
                        guideline_version: crate::GUIDELINE_VERSION.into(),
                        source: Source::Handwritten,
                        domain: p.domain,
                        hard_cases: p.hard_case.into_iter().collect(),
                        origin: Origin {
                            generator: None,
                            run_id: None,
                            group: None,
                        },
                    },
                }
            })
            .collect();
        let mut c =
            Composition::from_json(include_str!("../../../docs/gold-set/composition-v0.json"))
                .unwrap();
        // Elle yazılmış belgeler üretimden gelmez; ayrı eklenir.
        c.min_handwritten_share = 0.0;
        // Plan 650 belge; hedef 500 belge üzerinden, bu yüzden sayıları ölçekle.
        let stats = Stats::from_records(&records);
        let deviations = c.check(&stats);
        assert!(deviations.is_empty(), "{deviations:#?}");
        // Seyrek etiketlerde pay: 500 belgeye düşünce de 10'un üstünde kalsın.
        for &label in Label::ALL {
            let n = stats.label_spans.get(&label).copied().unwrap_or(0);
            assert!(n >= 16, "{label}: {n}");
        }
        assert!(stats.child_spans >= 30, "{}", stats.child_spans);
        assert!(
            stats.high_risk_documents >= 200,
            "{}",
            stats.high_risk_documents
        );
    }
}
