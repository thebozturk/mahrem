//! Ortak conformance seti: Rust ve Python aynı dosyayı okur.

use mahrem_core::{
    is_valid_card_number, is_valid_iban_tr, is_valid_tckn, is_valid_vkn, is_valid_ykn,
    to_lowercase_tr, to_uppercase_tr,
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct Case {
    func: String,
    input: String,
    expected: Value,
}

#[test]
fn validators_conformance() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../conformance/validators.jsonl"
    );
    let data = std::fs::read_to_string(path).expect("conformance dosyası okunamadı");
    let mut count = 0;
    for (line_no, line) in data
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty())
    {
        let case: Case = serde_json::from_str(line).expect("geçersiz JSON satırı");
        let got = match case.func.as_str() {
            "is_valid_tckn" => Value::Bool(is_valid_tckn(&case.input)),
            "is_valid_ykn" => Value::Bool(is_valid_ykn(&case.input)),
            "is_valid_vkn" => Value::Bool(is_valid_vkn(&case.input)),
            "is_valid_iban_tr" => Value::Bool(is_valid_iban_tr(&case.input)),
            "is_valid_card_number" => Value::Bool(is_valid_card_number(&case.input)),
            "to_lowercase_tr" => Value::String(to_lowercase_tr(&case.input)),
            "to_uppercase_tr" => Value::String(to_uppercase_tr(&case.input)),
            other => panic!("satır {}: bilinmeyen fonksiyon {other}", line_no + 1),
        };
        assert_eq!(
            got,
            case.expected,
            "satır {}: {}({:?})",
            line_no + 1,
            case.func,
            case.input
        );
        count += 1;
    }
    assert!(count > 0, "conformance seti boş");
}
