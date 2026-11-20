# Benchmark

`mahrem-bench` measures a PII detection tool against a gold set. It works with any tool: the tool writes a [prediction file](gold-set/README.md#prediction-record), and `mahrem-bench` scores it. Reports contain **numbers only**, never text or values. You can therefore run it on your own data and share the report.

## Commands

```sh
# Score a tool's predictions
mahrem-bench score --gold gold.jsonl --pred predictions.jsonl \
  [--scope all|structural|high-risk] [--min-score 0.5] [--tool "my-tool 1.2"] \
  [--json report.json] [--misses]

# Produce predictions with Mahrem's own structural recognizers, and measure latency
mahrem-bench predict --gold gold.jsonl --out mahrem.predictions.jsonl

# Validate gold and prediction files
mahrem-bench validate gold.jsonl [--pred predictions.jsonl]

# Distribution of a gold set, optionally against the composition targets
mahrem-bench stats gold.jsonl [--composition v0]
```

- **Several files:** `--gold` can be given more than once.
- **Validation first:** Files are validated before scoring, and an invalid gold or prediction file is refused. A gold document without a prediction record counts as "nothing found" and is reported.
- **Thresholds and latency:** `predict` writes every candidate with its score, so `score --min-score` can try different thresholds. Its latency figure (p95 on 2,000-character texts) is meaningful only for a release build: `cargo build --release -p mahrem-bench`.

## Metrics

| Metric | Definition |
| --- | --- |
| **Leak-free document rate** (main) | Of the documents with at least one gold span, the share where no span leaked |
| **High-risk leak-free rate** | The same, counting only high-risk spans: TCKN/YKN, VKN, IBAN, card numbers, KVKK Art. 6 |
| **False alarm rate** | Of the documents with no gold span, the share with at least one prediction |
| **Protection** (per label) | Share of gold spans that did not leak |
| **Exact recall / precision** (per label) | Same label and identical boundaries |
| **Overlap recall / precision** (per label) | Same label and at least one shared character |

**What "leaked" means.** A gold span is protected when every non-whitespace character in it is covered by some prediction, *whatever its label*:
- A phone number masked as `TCKN` still never reaches the LLM.
- A number with its last two digits uncovered has leaked.
- Masking digit groups one by one is fine; the spaces between them are not personal data.

**Why documents.** The main metric is per document because entity recall hides the risk. At 90% recall, a message with five pieces of personal data leaks at least one of them about 41% of the time (1 − 0.9⁵). Documents without personal data do not enter the leak-free rate, which would otherwise be inflated; they are measured by the false alarm rate instead.

**Scope.** `--scope` limits all metrics to a group of labels:
- `structural`: TCKN/YKN, VKN, IBAN, card numbers, phone numbers, e-mail. This is the v0.1 scope and the scope of Gate 1.
- `high-risk`;
- `all` (the default).

A tool that does not detect names should be measured with `--scope structural`; with `all`, every document with a name counts as leaked.

Predictions with labels outside the scope still protect the characters they cover, but they do not count for precision or false alarms.

**Thresholds.** Predictions with a score below `--min-score` (default 0.5) are ignored. Predictions without a score always count.

## Report

The terminal report and the `--json` report contain:
- the gold file names (not paths), their document counts and canonical digests;
- the prediction file name and SHA-256;
- the scope and threshold;
- the counts and rates above, per label and in total.

`--misses` lists leaked and extra spans with document ID, offsets and label only.

The open and closed parts of the gold set are scored separately; every release report gives both. To have a tool scored on the closed part, send its prediction file: the closed part's text never leaves the project, and prediction files do not contain text.

## Using your own data

1. Label your documents in the [gold set format](gold-set/README.md), following the [labeling guideline](gold-set/labeling-guideline.md).
2. Run `mahrem-bench validate` on them.
3. Produce predictions: `mahrem-bench predict` for Mahrem, or your own tool's output converted to the prediction format.
4. Run `mahrem-bench score --json report.json`. The report can be shared; your data stays with you.
