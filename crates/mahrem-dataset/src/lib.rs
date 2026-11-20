//! Altın set ve tahmin dosyaları.
//!
//! Altın set JSONL'dir: her satır bir belge ([`GoldRecord`]). Konumlar NFC
//! metin üzerinde Unicode code point cinsinden, bitiş hariçtir. Benchmark'a
//! verilen araç çıktısı aynı span yapısını kullanır ([`PredictionRecord`]).
//!
//! Format belgesi: `docs/gold-set/README.md`. Bu crate motor değil, veri
//! aracıdır; etiket tanımları [`mahrem_core::Label`]'dadır.

pub mod eval;
mod label_serde;
pub mod prediction;
pub mod record;
pub mod split;
pub mod stats;
pub mod validate;

pub use eval::{LabelCounts, Miss, Report, Scope, evaluate};
pub use prediction::{PredictedSpan, PredictionRecord};
pub use record::{Domain, GoldRecord, GoldSpan, HardCase, Meta, Origin, SCHEMA_VERSION, Source};
pub use split::{Part, canonical_jsonl, digest, part_of};
pub use stats::{Composition, Deviation, Range, Stats};
pub use validate::{GoldSet, Issue, IssueKind, Severity, check_record};
