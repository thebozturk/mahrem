//! Çekirdeğin sahte değer üreticileri doğrulayıcıdan uyarısız geçer: sentetik
//! veri hattının ürettiği span'ler yönergeye ve checksum'lara uyar.

use mahrem_core::Label;
use mahrem_core::fake::{Rng, generate};
use mahrem_dataset::{
    Domain, GoldRecord, GoldSpan, Meta, Origin, SCHEMA_VERSION, Source, check_record,
};

#[test]
fn generated_values_validate_cleanly() {
    for seed in 0..300 {
        let mut rng = Rng::new(seed);
        for &label in Label::ALL {
            let value = generate(label, &mut rng);
            let prefix = "Bilgi: ";
            let text = format!("{prefix}{value} olarak kayıtlı.");
            let start = prefix.chars().count();
            let rec = GoldRecord {
                id: format!("fake-{seed}-{}", label.id()),
                text,
                spans: vec![GoldSpan {
                    start,
                    end: start + value.chars().count(),
                    label,
                    text: value.clone(),
                    uncertain: false,
                    child: false,
                }],
                meta: Meta {
                    schema_version: SCHEMA_VERSION,
                    guideline_version: "0.2.0".into(),
                    source: Source::Template,
                    domain: Domain::Other,
                    hard_cases: vec![],
                    origin: Origin {
                        generator: None,
                        run_id: None,
                        group: None,
                    },
                },
            };
            let issues = check_record(&rec);
            assert!(issues.is_empty(), "{label} {value:?}: {issues:?}");
        }
    }
}
