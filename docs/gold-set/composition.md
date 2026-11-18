# Gold set composition

This document fixes what the gold set must look like before it is produced, so that its numbers can be trusted. The record format is in [README.md](README.md), the labeling rules in [labeling-guideline.md](labeling-guideline.md).

The exact targets for each version are in a machine-readable file, checked in tests and by the distribution report in [`mahrem-dataset`](../../crates/mahrem-dataset): [`composition-v0.json`](composition-v0.json).

## Two parts, one distribution

The gold set has an **open part**, published under CC BY 4.0 in [`datasets/`](../../datasets/README.md), and a **closed part** that is never published. Every release report gives both results. If the open part leaks into model training data, the closed part still measures honestly.

The two numbers are only comparable if both parts come from the same distribution. Therefore:

- Both parts are produced **by the same process, in the same run**, and then split. One part is never written by hand while the other is generated, or produced by a different model.
- The closed part's text must never reach an external service, so the process that produces it runs locally: hand-written text, templates, or a local open-weight model.
- Sets produced for other purposes are not part of the gold set, for example the [hard-cases set](../../datasets/README.md), whose texts were written by an external LLM to cover the guideline rule by rule.

## Size

v0 has **at least 500 documents per part**. The gold set's first job is the model decision: comparing candidate models whose differences are expected to be large.

For a leak-free document rate near 90%, the 95% confidence interval is about ±2.6 points with 500 documents and ±1.9 points with 1,000. Before the Gate 2 and v1.0 measurements, each part grows to at least 1,000 documents.

The high-risk target (≥ 99% leak-free) needs more. To show that the rate is above 99% when the true rate is 99.5%, about 765 documents containing high-risk data are needed. v0 asks for 150 (about ±1.6 points at 99%). The v1.0 composition raises this.

## Targets

`composition-v0.json` sets, per part:

| Field | Meaning | Why |
| --- | --- | --- |
| `documents_per_part` | Minimum number of documents | See *Size* |
| `entities_per_document` | Share of documents by number of entities: 0, 1, 2–3, 4–5, 6+ | The main metric is per document. A set of one-entity messages would overstate it; documents with many entities are where leaks happen |
| `domains` | Share range for each text type | Customer support and chat are the main use; logs and documents behave differently |
| `min_spans_per_label` | Minimum spans for every label | Every label gets a per-category number, even rare special categories |
| `min_high_risk_documents` | Minimum documents with at least one high-risk span | See *Size* |
| `min_child_spans` | Minimum spans with `child: true` | Children's data is reported separately |
| `min_hard_case_share` | Minimum share of documents with at least one hard case | Irregular writing is common in real messages |
| `min_handwritten_share` | Minimum share of hand-written documents | Generated text alone has its own regularities |

About 20% of documents contain no personal data (the `0` bucket), so false alarms are measured too.

## Split

The split is deterministic and works on **groups**, not documents. Documents from the same template or the same prompt share `meta.origin.group` and must land in the same part; otherwise the open part would reveal the closed part's templates. A document without a group is its own group.

For a seed `s` and a group `g`, take the first 8 bytes of `SHA-256(s + "\n" + g)` as a big-endian integer and divide by 2⁶⁴. If the result is below the open share (0.5 for v0), the group is open; otherwise it is closed.

```python
x = int.from_bytes(hashlib.sha256((seed + "\n" + group).encode()).digest()[:8], "big") / 2**64
part = "open" if x < open_share else "closed"
```

The seed is chosen once per version and published with the release. After the split, each part is checked against the targets separately.

## Closed part

- It is stored outside the repository, under `data/` (ignored by git), with the file name ending in `.closed.jsonl`. It is encrypted at rest and backed up. It is never sent to an external service, including LLM APIs.
- Each release publishes the SHA-256 digest of the closed part, so it cannot be changed silently. The digest covers a canonical form, reproducible in any language:
  1. Sort the records by `id`, comparing UTF-8 bytes.
  2. Write each record as JSON with keys sorted, no whitespace and non-ASCII characters unescaped, followed by a line feed.
  3. Hash the UTF-8 bytes with SHA-256 and write the result in lowercase hexadecimal.

  ```python
  canon = "".join(
      json.dumps(r, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n" for r in records
  )
  ```

- At every major release, the old closed part is opened (added to the open part) and a new closed part is produced. Published digests let anyone check the opened part against earlier reports.

## Contamination

Gold set texts never enter training data. Before training, generated training data is deduplicated against both parts of the gold set; the check is part of data generation.
