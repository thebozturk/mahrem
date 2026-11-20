# Changelog

All notable changes are listed here. Versions follow [semantic versioning](https://semver.org/). Before 1.0, minor versions may change the API. Deprecations are announced in a release before the removal.

## [0.0.1]

First release, published to claim the package names with working code. Gate 1 (v0.1) has not been reached yet; see the [roadmap](ROADMAP.md).

### Added

- `analyze`: structural detection of TCKN/YKN, VKN, IBAN, payment card numbers, phone numbers and e-mail addresses in free text, including spaced, hyphenated and line-broken numbers and written-out e-mail addresses. Rust (`mahrem-core`) and Python (`mahrem`).
- Checksum validators for TCKN, YKN, VKN, Turkish IBAN and card numbers.
- Turkish-aware case mapping (`İ` ↔ `i`, `I` ↔ `ı`).
- Labels, including the KVKK Art. 6 special categories.
- Seeded fake value generators with valid checksums (Rust).
