# Structural detection

`analyze` finds six kinds of structured personal data in Turkish text: TCKN/YKN, VKN, IBAN, payment card numbers, phone numbers and e-mail addresses. It is the v0.1 detection layer. Names, addresses and the other categories come with the NER layer in v0.5.

## Usage

```rust
use mahrem_core::{Label, analyze};

let text = "TCKN: 10000000146, tel 0532 123 45 67";
for e in analyze(text) {
    println!("{} {}..{} {:?} {:.2} {}", e.label, e.start, e.end, e.text(text), e.score, e.source);
}
// TCKN 6..17 "10000000146" 0.99 tckn
// PHONE 23..37 "0532 123 45 67" 0.99 phone
```

```python
import mahrem

for e in mahrem.analyze("TCKN: 10000000146, tel 0532 123 45 67"):
    print(e.label, e.start, e.end, e.text, round(e.score, 2), e.source)
```

- **Offsets** are Unicode code points, end-exclusive: the same as Python `str` indices and the [gold set format](gold-set/README.md).
- **Results** are sorted by start and never overlap.
- **Threshold:** `analyze(text)` uses the default threshold of 0.5. `analyze_with(text, min_score)` in Rust, or `analyze(text, min_score=…)` in Python, changes it.

## What is detected

| Label | Rule | Score | `source` |
| --- | --- | --- | --- |
| `TCKN` | 11 digits, first not 0, TCKN checksum | 0.95; 0.99 with context | `tckn` |
| `YKN` | Same checksum, starts with `99` | 0.95; 0.99 with context | `ykn` |
| `VKN` | 10 digits, VKN checksum | **0.4**; 0.95 with context | `vkn` |
| `IBAN` | `TR` + 24 digits, mod 97; `tr` in lower case too | 0.99 | `iban` |
| `CARD_NUMBER` | Card network prefix and length (Visa, Mastercard, Amex, Troy, Discover, JCB, UnionPay) + Luhn | 0.9; 0.99 with context | `card` |
| `PHONE` | Turkish mobile (`05xx`, `+90 5xx`, bare `5xx` with 10 digits), landline (`0` + area code), `0850`/`0800`, `444` numbers, international numbers with `+` | 0.6–0.85; +0.15 with context | `phone` |
| `EMAIL` | `local@domain.tld` | 0.99 | `email` |
| `EMAIL` | Written out: `ayse at example nokta com`, `ayse[at]example[nokta]com`, `(at)`, `dot` | 0.9 | `email-obfuscated` |

**Context words** are matched by prefix, in Turkish and English, up to five words before and three words after the number:
- TCKN/YKN: *tc, kimlik, ykn*;
- VKN: *vergi, vkn, v.d.*;
- card: *kart, kredi, visa*;
- phone: *tel, telefon, cep, gsm, numara*.

**VKN:** About one in ten random 10-digit numbers passes the VKN checksum, so order and customer numbers would be flagged. Without a context word a VKN gets 0.4, below the default threshold. With a context word before or after it ("VKN: …", "… vergi numarasıyla") it gets 0.95.

## Irregular writing

A number may be broken up by spaces, hyphens, dots, slashes or parentheses, up to four characters between digit groups, and by one line break:

| Written as | Found as |
| --- | --- |
| `0 5 5 5 - 0 0 0 - 0 0 - 0 4` | `PHONE` |
| `+90 (555) 000 00 01`, `(0212) 000 00 02` | `PHONE`, including `+` and the area-code parentheses |
| `TR33 0006 1005⏎1978 6457 8413 26` | `IBAN`, including the line break |
| `4111-1111-1111-1111` | `CARD_NUMBER` |

**Adjacent numbers:** Candidates are tried only at digit-group boundaries, so numbers written next to each other stay apart. For example, `kart 4111 1111 1111 1111 TC 10000000146` gives a card number and a TCKN.

**Not treated as numbers:**
- digits glued to letters, such as codes like `TK000111222` or `sk-example-0000`;
- dot-separated groups of at most three digits: IP addresses, version numbers, amounts like `1.250.000`.

## Overlaps

When candidates overlap, the longest one wins; if they are equally long, the higher score wins. For example, an 11-digit window inside a valid card number is not reported as a TCKN.

## Known limits

- **Numbers written in words** (*sıfır beş yüz otuz iki*) are not detected; they come in v0.5.
- **IBANs** are detected only for Turkey.
- **Phone numbers** have no checksum. A 10- or 11-digit number with a mobile or area-code prefix is reported, even if it is something else.
- **Landline numbers** are detected only with a leading `0` or `+90`.
- **International numbers** are detected only with a leading `+`.
- **Suffixes:** Spans end at the last digit or letter of the value. Suffixes stay outside (`0555 000 00 07'ye`), as the [labeling guideline](gold-set/labeling-guideline.md) requires.
