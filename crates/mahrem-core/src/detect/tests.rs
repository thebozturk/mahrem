use super::*;

/// Bulunanlar: (etiket kimliği, metin).
fn found(text: &str) -> Vec<(&'static str, String)> {
    analyze(text)
        .iter()
        .map(|e| (e.label.id(), e.text(text).to_owned()))
        .collect()
}

fn one(text: &str, label: &str, value: &str) {
    assert_eq!(
        found(text),
        vec![(label_static(label), value.to_owned())],
        "{text:?}"
    );
}

fn label_static(id: &str) -> &'static str {
    Label::ALL
        .iter()
        .find(|l| l.id() == id)
        .expect("etiket")
        .id()
}

#[test]
fn identity_numbers() {
    one("TCKN: 10000000146.", "TCKN", "10000000146");
    one(
        "kimlik numaram 3 0 0 0 0 0 0 0 3 2 8 yazıyor",
        "TCKN",
        "3 0 0 0 0 0 0 0 3 2 8",
    );
    one(
        "Yabancı kimlik no: 99000000042, uyruğum Alman.",
        "YKN",
        "99000000042",
    );
    one("VKN: 123-456-7890", "VKN", "123-456-7890");
    one(
        "Faturayı 1234567890 vergi numarasıyla kesin.",
        "VKN",
        "1234567890",
    );
}

#[test]
fn vkn_without_context_is_below_threshold() {
    assert!(found("Sipariş 1234567890 kargoya verildi.").is_empty());
    let low = analyze_with("Sipariş 1234567890 kargoya verildi.", 0.3);
    assert_eq!(low.len(), 1);
    assert_eq!(low[0].label, Label::Vkn);
    assert!(low[0].score < DEFAULT_MIN_SCORE);
}

#[test]
fn iban() {
    one(
        "İadeyi TR33 0006 1005 1978 6457 8413 26 hesabına yapın.",
        "IBAN",
        "TR33 0006 1005 1978 6457 8413 26",
    );
    one(
        "IBAN:\nTR33 0006 1005\n1978 6457 8413 26\nAlıcı: x",
        "IBAN",
        "TR33 0006 1005\n1978 6457 8413 26",
    );
    one(
        "iban tr330006100519786457841326 ama",
        "IBAN",
        "tr330006100519786457841326",
    );
    one(
        "IBAN: TR 33 0006 1005 1978 6457 8413 26",
        "IBAN",
        "TR 33 0006 1005 1978 6457 8413 26",
    );
    assert!(
        found("TR330006100519786457841327").is_empty(),
        "checksum tutmayan IBAN"
    );
}

#[test]
fn cards() {
    one(
        "kart numaram 5555 5555 5555 4444, son kullanma 12/29",
        "CARD_NUMBER",
        "5555 5555 5555 4444",
    );
    one(
        "Ekstrede 4111-1111-1111-1111 görünüyor; son dört hanesi 4444.",
        "CARD_NUMBER",
        "4111-1111-1111-1111",
    );
    one(
        "Kart:\n4012 8888\n8888 1881",
        "CARD_NUMBER",
        "4012 8888\n8888 1881",
    );
    assert!(
        found("Numara 1234 5678 9012 3456").is_empty(),
        "kart ağı öneki yok"
    );
}

#[test]
fn phones() {
    let text = "Bana +90 (555) 000 00 01 ya da (0212) 000 00 02 sabit hattan ulaşın.";
    assert_eq!(
        found(text),
        vec![
            ("PHONE", "+90 (555) 000 00 01".to_owned()),
            ("PHONE", "(0212) 000 00 02".to_owned())
        ]
    );
    one(
        "Numaram 0 5 5 5 - 0 0 0 - 0 0 - 0 4, akşam arayın.",
        "PHONE",
        "0 5 5 5 - 0 0 0 - 0 0 - 0 4",
    );
    one("Çağrı merkezi 444 0 000 numarası", "PHONE", "444 0 000");
    one("0555 000 00 07'ye yazın", "PHONE", "0555 000 00 07");
    one("tel:05321234567", "PHONE", "05321234567");
    one("Destek 0850 123 45 67", "PHONE", "0850 123 45 67");
    one("Londra +44 20 7946 0958", "PHONE", "+44 20 7946 0958");
    one("(0532 123 45 67)", "PHONE", "0532 123 45 67");
}

#[test]
fn emails() {
    let text =
        "Belgeleri derya.kaplan@example.com adresine, kopyasını destek@example.org adresine.";
    assert_eq!(
        found(text),
        vec![
            ("EMAIL", "derya.kaplan@example.com".to_owned()),
            ("EMAIL", "destek@example.org".to_owned())
        ]
    );
    one(
        "OKAN.CELIK@EXAMPLE.COM ADRESİME YAZIN",
        "EMAIL",
        "OKAN.CELIK@EXAMPLE.COM",
    );
    one(
        "mailim mertyilmaz at example nokta net, oradan yazın",
        "EMAIL",
        "mertyilmaz at example nokta net",
    );
    one(
        "ayse[at]example[nokta]com.tr adresi",
        "EMAIL",
        "ayse[at]example[nokta]com.tr",
    );
    one("ayse (at) example.com", "EMAIL", "ayse (at) example.com");
    one("Adres: ece@example.com.", "EMAIL", "ece@example.com");
    assert!(found("@deniz_kaya bugün geldi; saat 10 at koşusu").is_empty());
}

#[test]
fn adjacent_numbers_stay_apart() {
    assert_eq!(
        found("kart 4111 1111 1111 1111 TC 10000000146"),
        vec![
            ("CARD_NUMBER", "4111 1111 1111 1111".to_owned()),
            ("TCKN", "10000000146".to_owned())
        ]
    );
    assert_eq!(
        found("0532 123 45 67 10000000146"),
        vec![
            ("PHONE", "0532 123 45 67".to_owned()),
            ("TCKN", "10000000146".to_owned())
        ]
    );
}

#[test]
fn no_false_alarms() {
    for text in [
        "Doğum tarihi: 04.07.1988",
        "2026-11-18T10:22:01Z WARN ip=198.51.100.23 token=sk-example-0000000000000000",
        "Kargo takip kodum TK000111222, sipariş 1000234.",
        "Abone numaram 4000123456.",
        "Toplam 1.250.000,00 TL ödendi; 12.500.000.000 TL bütçe.",
        "Hasta protokol no: 2026/00042",
        "Sürüm 10.100.200.150 yayında.",
        "Saat 14:30'da, 3 kişi, 25 TL.",
    ] {
        assert!(found(text).is_empty(), "{text:?}: {:?}", found(text));
    }
}

#[test]
fn offsets_are_code_points() {
    let text = "Öğrenci İçin: 10000000146";
    let e = &analyze(text)[0];
    assert_eq!((e.start, e.end), (14, 25));
    assert_eq!(e.text(text), "10000000146");
    assert_eq!(e.source, "tckn");
}

#[test]
fn results_are_sorted_and_disjoint() {
    let text = "a@b.co 0532 123 45 67 TR330006100519786457841326 10000000146 4111111111111111";
    let r = analyze(text);
    assert_eq!(r.len(), 5);
    for w in r.windows(2) {
        assert!(w[0].end <= w[1].start);
    }
}

mod props {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn never_panics_and_stays_in_bounds(s in "\\PC{0,200}") {
            let n = s.chars().count();
            let r = analyze_with(&s, 0.0);
            for e in &r {
                prop_assert!(e.start < e.end && e.end <= n);
                prop_assert!((0.0..=1.0).contains(&e.score));
            }
            for w in r.windows(2) {
                prop_assert!(w[0].end <= w[1].start);
            }
        }

        #[test]
        fn digit_soup_is_safe(s in "[0-9 .\\-/()\\n+@a-zTR]{0,300}") {
            let r = analyze_with(&s, 0.0);
            for e in &r {
                prop_assert!(e.end <= s.chars().count());
            }
        }
    }
}
