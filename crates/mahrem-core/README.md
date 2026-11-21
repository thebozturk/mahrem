# mahrem-core

The Rust core of [Mahrem](https://github.com/thebozturk/mahrem): a privacy layer for LLMs, built for Turkish. Mahrem masks personal data in Turkish text before it reaches an LLM and restores the real values, with correct Turkish suffixes, in the reply.

This early release contains the structural detection layer. Masking and restoration come in later versions.

- **Structural recognizers:** TCKN/YKN, VKN, IBAN, payment card numbers, phone numbers and e-mail addresses in free text, including spaced, hyphenated and line-broken numbers and written-out e-mail addresses.
- **Checksum validators** for Turkish identity, tax and bank numbers.
- **Turkish-aware case mapping:** `İ` ↔ `i`, `I` ↔ `ı`.
- **Labels:** the label set, including the KVKK Art. 6 special categories.
- **Fake value generators:** seeded and reproducible, with valid checksums.

```rust
use mahrem_core::{Label, analyze, is_valid_tckn, to_lowercase_tr};

let found = analyze("TCKN: 10000000146, tel 0 5 3 2 - 1 2 3 - 4 5 - 6 7");
assert_eq!(found[0].label, Label::Tckn);
assert_eq!(found[1].label, Label::Phone);

assert!(is_valid_tckn("10000000146"));
assert_eq!(to_lowercase_tr("İSTANBUL ILGAZ"), "istanbul ılgaz");
```

Offsets are Unicode code points, end-exclusive. Details: [detection](https://github.com/thebozturk/mahrem/blob/main/docs/detection.md).

Mahrem is pseudonymization, not anonymization: whoever holds the mapping can reverse it.

The API is not stable before 1.0. Licensed under Apache-2.0.
