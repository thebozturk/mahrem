# Datasets

Open Mahrem datasets. Everything in this folder is published under [CC BY 4.0](LICENSE); the code in the rest of the repository is Apache-2.0.

| Set | File | Documents | Content |
| --- | --- | --- | --- |
| Hard cases | [`hard-cases/hard-cases.jsonl`](hard-cases/hard-cases.jsonl) | 82 | At least one document per labeling guideline rule and per label; irregular spelling, ambiguous names, spelled and split numbers; documents with no personal data |

Planned: the open part of the gold set. Its closed part is never stored here.

## Format and labels

Every file follows the [gold set format](../docs/gold-set/README.md) and the [labeling guideline](../docs/gold-set/labeling-guideline.md) version given in each record's `meta.guideline_version`. CI validates every `.jsonl` file in this folder together: no errors, no warnings, unique IDs across files.

## Provenance

- **All values are synthetic.** ID numbers are generated with valid checksums from obvious patterns, card numbers are card networks' test numbers, e-mail addresses use `example.com`/`.org`/`.net`, IP addresses come from documentation ranges (RFC 5737, RFC 7042 for MAC). Names are made up; any match with a real person is a coincidence.
- **Who wrote it** is recorded in each record's `meta.source` and `meta.origin.generator`. The hard-cases texts and their first labels were written by an LLM (`claude-opus-5-5`).
- **Labels are human-approved.** A record enters this folder only through a reviewed pull request; review is the human approval step of the labeling workflow.

## Contributing

Corrections are welcome as pull requests or issues. Only synthetic values are accepted ([CONTRIBUTING.md](../CONTRIBUTING.md)). If a correction changes how a rule is applied, the labeling guideline changes in the same pull request.
