# mahrem

**A privacy layer for LLMs, built for Turkish.** Mahrem masks personal data in Turkish text before it reaches an LLM and restores the real values, with correct Turkish suffixes, in the reply.

This early release contains the structural detection layer. Masking and restoration come in later versions; see the [roadmap](https://github.com/thebozturk/mahrem/blob/main/ROADMAP.md).

```sh
pip install mahrem
```

## Find personal data

```python
import mahrem

for e in mahrem.analyze("TCKN: 10000000146, tel 0 5 3 2 - 1 2 3 - 4 5 - 6 7"):
    print(e.label, e.start, e.end, e.text, round(e.score, 2))
# TCKN 6 17 10000000146 0.99
# PHONE 23 50 0 5 3 2 - 1 2 3 - 4 5 - 6 7 0.99
```

`analyze` finds TCKN/YKN, VKN, IBAN, payment card numbers, phone numbers and e-mail addresses, including spaced, hyphenated and line-broken numbers and written-out addresses (`ayse at example nokta com`).
- **Offsets** are Python `str` indices.
- **`min_score`** sets the threshold; the default is 0.5.

Details: [detection](https://github.com/thebozturk/mahrem/blob/main/docs/detection.md).

## Validators and case mapping

```python
mahrem.is_valid_tckn("10000000146")  # True
mahrem.is_valid_iban_tr("TR330006100519786457841326")  # True
mahrem.to_lowercase_tr("İSTANBUL ILGAZ")  # 'istanbul ılgaz', not 'i̇stanbul ilgaz'
```

Also available: `is_valid_ykn`, `is_valid_vkn`, `is_valid_card_number` and `to_uppercase_tr`.

## About

- The engine is written in Rust; this package is a thin binding.
- The same core runs the benchmark CLI and the planned gateway, so every surface gives the same result.
- Mahrem is pseudonymization, not anonymization: whoever holds the mapping can reverse it.
- No telemetry.
- The API is not stable before 1.0.

[Source and documentation](https://github.com/thebozturk/mahrem) · Apache-2.0
