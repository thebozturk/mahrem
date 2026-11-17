# Gold set format

This document defines the file format of the Mahrem gold set: the human-verified Turkish test set that Mahrem and other tools are measured on. It also defines the format a tool's output must have to be scored by the benchmark.

- **Schema version:** 1
- **Validator:** [`crates/mahrem-dataset`](../../crates/mahrem-dataset) (Rust). The examples in this folder are validated in CI.
- **Examples:** [`example.jsonl`](example.jsonl) (gold), [`example.predictions.jsonl`](example.predictions.jsonl) (tool output). All values are synthetic.

What to label, and where a span starts and ends, is decided by the [labeling guideline](labeling-guideline.md) (in Turkish), not by this document. The format only makes those decisions machine-checkable.

## Files

The gold set is [JSON Lines](https://jsonlines.org/): UTF-8, one document per line. The open part is published under CC BY 4.0. The closed part has the same format, is never published, and its files end in `.closed.jsonl`. A document's split is given by the file it is in, not by a field.

Documents that contain no personal data are part of the set, with an empty `spans` list. They measure false alarms.

## Gold record

```json
{
  "id": "example-0001",
  "text": "Merhaba, ben Ayşe Demir. Kızım Elif'in astımı var, 7 yaşında. Bana 0555 000 00 00 numarasından ulaşabilirsiniz.",
  "spans": [
    {"start": 13, "end": 23, "label": "PERSON", "text": "Ayşe Demir", "uncertain": false, "child": false},
    {"start": 31, "end": 35, "label": "PERSON", "text": "Elif", "uncertain": false, "child": true},
    {"start": 39, "end": 44, "label": "HEALTH", "text": "astım", "uncertain": false, "child": false},
    {"start": 51, "end": 52, "label": "AGE", "text": "7", "uncertain": false, "child": true},
    {"start": 67, "end": 81, "label": "PHONE", "text": "0555 000 00 00", "uncertain": false, "child": false}
  ],
  "meta": {
    "schema_version": 1,
    "guideline_version": "0.1.0",
    "source": "handwritten",
    "domain": "customer_support",
    "hard_cases": [],
    "origin": {"generator": null, "run_id": null}
  }
}
```

(Shown on several lines for reading; in the file each record is a single line.)

| Field | Type | Meaning |
| --- | --- | --- |
| `id` | string | Unique across all gold files, open and closed. `[A-Za-z0-9._-]`, 1–128 characters. |
| `text` | string | The document, in Unicode NFC. Not empty. |
| `spans` | array | Personal data in the document, sorted by `start`, non-overlapping. |
| `meta` | object | See below. |

Every field is required and unknown fields are rejected, so every file has the same columns (Arrow, Parquet and Hugging Face `datasets` infer one schema).

### Span

| Field | Type | Meaning |
| --- | --- | --- |
| `start` | integer | Start offset, in Unicode code points. |
| `end` | integer | End offset, exclusive. |
| `label` | string | Label ID, from the table below. |
| `text` | string | `text[start:end]`, repeated so that offsets can be checked by eye and by code. |
| `uncertain` | bool | The annotator was not sure. The guideline says: when in doubt, label and mark it uncertain. |
| `child` | bool | The value belongs to a child. Allowed only on `PERSON` and `AGE`. |

### Offsets

Offsets count **Unicode code points** in the NFC text, with an **exclusive end**. In Python, `record["text"][span["start"]:span["end"]] == span["text"]` holds for every span, with no conversion. This is also how Hugging Face `datasets` and spaCy count characters.

- Rust and other UTF-8 byte-indexed languages must convert. Mahrem's core provides `CodePointIndex` for this.
- JavaScript strings are UTF-16: characters outside the Basic Multilingual Plane (emoji, for example) count as two there, but as one here. Use `Array.from(text)` to index by code point.
- Text must be NFC because the same visible text can have different code point lengths in different normalization forms.

### Meta

| Field | Type | Meaning |
| --- | --- | --- |
| `schema_version` | integer | `1` for this document. |
| `guideline_version` | string | Version of the labeling guideline the labels follow, `X.Y.Z`. |
| `source` | string | How the text was written: `handwritten`, `template` (templates filled by value generators) or `llm` (an LLM wrote the text with placeholders, value generators filled them). |
| `domain` | string | `customer_support`, `email`, `chat`, `form`, `document`, `social_media`, `log` or `other`. |
| `hard_cases` | array | Patterns in this document that are known to defeat rules or models (see below). Empty for ordinary documents. |
| `origin` | object | Provenance: `generator` (model name or template ID, `null` for handwritten text) and `run_id` (the batch run that produced it, or `null`). |

Hard case values:

| Value | Example |
| --- | --- |
| `no_diacritics` | *Ayse Yilmaz* for *Ayşe Yılmaz* |
| `all_caps` | *AYŞE YILMAZ* |
| `all_lowercase` | *ayşe yılmaz* |
| `typo` | Misspelled names or words around the data |
| `ambiguous_name` | Names that are also common words: *Umut* (hope), *Deniz* (sea), *Bahar* (spring) |
| `name_in_place` | A person's name inside a street or organization name: *Atatürk Caddesi* |
| `spelled_numbers` | *sıfır beş yüz otuz iki* |
| `spaced_numbers` | *0 5 3 2 - 1 2 3* |
| `split_lines` | An IBAN or card number broken across lines |
| `obfuscated_email` | *ayse at example nokta com* |

## Labels

Each label has a permanent ASCII **ID**, used in data files and APIs, and a default Turkish **placeholder name**, used in the masked text sent to an LLM (`<KİŞİ_1>`). The placeholder name can be changed by policy; the ID never changes.

**High risk** labels are reported separately: their target leak-free document rate is higher.

| ID | Placeholder | Covers | High risk |
| --- | --- | --- | --- |
| `TCKN` | `TCKN` | Turkish national ID number | ✓ |
| `YKN` | `YKN` | Foreign resident ID number (starts with 99) | ✓ |
| `VKN` | `VKN` | Tax ID number | ✓ |
| `IBAN` | `IBAN` | IBAN | ✓ |
| `CARD_NUMBER` | `KART` | Payment card number | ✓ |
| `PHONE` | `TELEFON` | Mobile or landline number | |
| `EMAIL` | `E_POSTA` | E-mail address | |
| `PASSPORT` | `PASAPORT` | Passport number | |
| `DRIVER_LICENSE` | `EHLİYET` | Driver's license number | |
| `SGK_NUMBER` | `SGK_NO` | Social security (SGK) registration number | |
| `MERSIS` | `MERSİS` | Central business registry (MERSİS) number | |
| `LICENSE_PLATE` | `PLAKA` | Vehicle license plate | |
| `SERIAL_NUMBER` | `SERİ_NO` | Device serial number | |
| `IP_ADDRESS` | `IP` | IP address | |
| `MAC_ADDRESS` | `MAC` | MAC address | |
| `DATE` | `TARİH` | A date tied to a person, such as a date of birth | |
| `SECRET` | `GİZLİ` | API key, JWT, private key, password | |
| `PERSON` | `KİŞİ` | Person's name | |
| `ADDRESS` | `ADRES` | Postal address; a contiguous address is one span | |
| `ORGANIZATION` | `KURUM` | Employer, school, hospital and other organizations | |
| `AGE` | `YAŞ` | Age | |
| `ETHNIC_ORIGIN` | `ETNİK_KÖKEN` | Race and ethnic origin | ✓ |
| `POLITICAL_OPINION` | `SİYASİ_GÖRÜŞ` | Political opinion | ✓ |
| `BELIEF` | `İNANÇ` | Philosophical belief, religion, sect and other beliefs | ✓ |
| `ATTIRE` | `KIYAFET` | Dress and attire | ✓ |
| `MEMBERSHIP` | `ÜYELİK` | Membership of an association, foundation or trade union | ✓ |
| `HEALTH` | `SAĞLIK` | Health | ✓ |
| `SEX_LIFE` | `CİNSEL_HAYAT` | Sex life | ✓ |
| `CRIMINAL_RECORD` | `CEZA_KAYDI` | Criminal convictions and security measures | ✓ |
| `BIOMETRIC` | `BİYOMETRİK` | Biometric data | ✓ |
| `GENETIC` | `GENETİK` | Genetic data | ✓ |

The last ten labels are the special categories of personal data in [KVKK](https://kvkk.gov.tr/Icerik/6649/Personal-Data-Protection-Law) Article 6(1), Türkiye's data protection law. The law lists thirteen items. Items that the law lists together, and that annotators could not reliably tell apart, share a label:

| KVKK Art. 6(1) item | Label |
| --- | --- |
| ırk (race), etnik köken (ethnic origin) | `ETHNIC_ORIGIN` |
| siyasi düşünce (political opinion) | `POLITICAL_OPINION` |
| felsefi inanç (philosophical belief), din (religion), mezhep veya diğer inançlar (sect or other beliefs) | `BELIEF` |
| kılık ve kıyafet (dress and attire) | `ATTIRE` |
| dernek, vakıf ya da sendika üyeliği (association, foundation or trade union membership) | `MEMBERSHIP` |
| sağlık (health) | `HEALTH` |
| cinsel hayat (sex life) | `SEX_LIFE` |
| ceza mahkûmiyeti ve güvenlik tedbirleri (criminal convictions and security measures) | `CRIMINAL_RECORD` |
| biyometrik veri (biometric data) | `BIOMETRIC` |
| genetik veri (genetic data) | `GENETIC` |

A special-category span covers the shortest expression that reveals the information (*astım*, asthma), not the whole sentence.

Children's data is not a separate label. A child's name is a `PERSON` and a child's age is an `AGE`, with `"child": true`.

## Prediction record

A tool's output is a JSONL file with one record per gold document. It does not repeat the document text, so it can be produced for the closed part without copying that text anywhere.

```json
{"id": "example-0001", "spans": [{"start": 13, "end": 23, "label": "PERSON", "score": 0.98}]}
```

| Field | Type | Meaning |
| --- | --- | --- |
| `id` | string | ID of the gold document. |
| `spans` | array | Spans found by the tool. Any order; overlaps are allowed. Empty if the tool found nothing. |
| `spans[].start`, `spans[].end`, `spans[].label` | | As in the gold record. |
| `spans[].text` | string, optional | If given, must equal the gold text at those offsets. |
| `spans[].score` | number, optional | Confidence in `[0, 1]`. |

Tools with their own label names must map them to the IDs above before scoring.

## Validation

**Errors** make a record unusable:

- Invalid JSON, a missing or unknown field, an unknown label.
- `id` invalid, or repeated in any of the files validated together.
- `schema_version` other than 1, or `guideline_version` not `X.Y.Z`.
- `text` empty or not NFC.
- A span with `start >= end`, beyond the end of the text, or whose `text` does not match the document.
- A span that starts or ends with whitespace.
- Spans not sorted by `start`, or overlapping (labeling is flat: only the outermost entity is labeled).
- `child: true` on a label other than `PERSON` or `AGE`.

**Warnings** point at likely guideline violations for a person to check:

- A span ending in an apostrophe and suffix (`Ahmet'in`). Suffixes stay outside the span.
- A `PERSON` span beginning or ending with an honorific, title or kinship term (`Hanım`, `Bey`, `Sayın`, `Dr.`, `Av.`, `Teyze`, `Hoca` and similar). These stay outside the span.
- A span beginning with an opening quote or bracket, or ending with a closing quote, bracket, comma, semicolon, colon, `!` or `?`. Punctuation around a value stays outside; periods are allowed because abbreviations (`Ltd. Şti.`) belong to the value.
- A `TCKN`, `YKN`, `VKN`, `IBAN` (Turkish) or `CARD_NUMBER` value that fails its checksum once spaces and `-./` are removed. Values that are not digits, such as spelled-out numbers, are skipped.
- A `TCKN` value that starts with 99, which makes it a `YKN`.
- A labeled value that appears again in the same document, at word boundaries, without a label.

For predictions, a missing record for a gold document is a warning; an unknown `id`, invalid offsets, a mismatching `text` or a score outside `[0, 1]` is an error.

## Versioning

A change that makes old files invalid or changes the meaning of a field increments `schema_version`. Label IDs are never renamed or reused; a removed label stays reserved.
