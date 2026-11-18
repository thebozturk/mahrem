//! Kanonik biçim, özet ve bölme başka dillerde de aynı sonucu vermeli.
//! Beklenen değerler Python ile, bu crate'ten bağımsız olarak hesaplandı:
//!
//! ```python
//! recs.sort(key=lambda r: r["id"].encode())
//! canon = "".join(json.dumps(r, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n" for r in recs)
//! hashlib.sha256(canon.encode()).hexdigest()
//! int.from_bytes(hashlib.sha256((seed + "\n" + group).encode()).digest()[:8], "big") / 2**64
//! ```

use mahrem_dataset::{GoldRecord, Part, digest, part_of};

#[test]
fn digest_matches_python() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/gold-set/example.jsonl"
    );
    let records: Vec<GoldRecord> = std::fs::read_to_string(path)
        .expect("example.jsonl okunamadı")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("geçersiz kayıt"))
        .collect();
    assert_eq!(
        digest(&records),
        "ec4e1b40cbf0b56d16f56951161535f4235ab36e38fb108c90aad4849343cce0"
    );
}

#[test]
fn split_matches_python() {
    // (grup, Python'daki oran): 0.115, 0.295, 0.686, 0.330
    let cases = [
        ("tpl-1", Part::Open, Part::Open),
        ("tpl-2", Part::Open, Part::Closed),
        ("example-0003", Part::Closed, Part::Closed),
        ("şablon-ç", Part::Open, Part::Closed),
    ];
    for (group, at_half, at_quarter) in cases {
        assert_eq!(part_of("mahrem-v0", group, 0.5), at_half, "{group}");
        assert_eq!(part_of("mahrem-v0", group, 0.25), at_quarter, "{group}");
    }
}
