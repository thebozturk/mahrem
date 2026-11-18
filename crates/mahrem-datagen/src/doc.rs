//! Belge parçaları ve kayda dönüştürülmesi.

use mahrem_core::Label;
use mahrem_dataset::{
    Domain, GoldRecord, GoldSpan, HardCase, Meta, Origin, SCHEMA_VERSION, Source, check_record,
};

/// Belgenin bir parçası: düz metin ya da etiketli değer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seg {
    /// Metin.
    pub text: String,
    /// Etiketli değerse etiket ve çocuk işareti.
    pub span: Option<(Label, bool)>,
}

impl Seg {
    /// Düz metin.
    pub fn plain(text: impl Into<String>) -> Self {
        Seg {
            text: text.into(),
            span: None,
        }
    }
}

/// Kaydın kökeni ve türü.
#[derive(Debug, Clone)]
pub struct RecordMeta<'a> {
    /// Metin türü.
    pub domain: Domain,
    /// Zor durumlar.
    pub hard_cases: Vec<HardCase>,
    /// Metnin kaynağı.
    pub source: Source,
    /// Metni yazan model.
    pub generator: Option<&'a str>,
    /// Bölme grubu.
    pub group: Option<String>,
}

/// Parçalardan kaydı kurar ve altın set doğrulayıcısından geçirir.
///
/// # Errors
///
/// Doğrulayıcı herhangi bir bulgu verirse; gerekçe `dogrulayici-<tür>`.
pub fn record(id: &str, segs: &[Seg], meta: RecordMeta<'_>) -> Result<GoldRecord, String> {
    let mut text = String::new();
    let mut spans = Vec::new();
    let mut pos = 0;
    for seg in segs {
        let len = seg.text.chars().count();
        if let Some((label, child)) = seg.span {
            spans.push(GoldSpan {
                start: pos,
                end: pos + len,
                label,
                text: seg.text.clone(),
                uncertain: false,
                child,
            });
        }
        text.push_str(&seg.text);
        pos += len;
    }
    let rec = GoldRecord {
        id: id.to_owned(),
        text,
        spans,
        meta: Meta {
            schema_version: SCHEMA_VERSION,
            guideline_version: crate::GUIDELINE_VERSION.to_owned(),
            source: meta.source,
            domain: meta.domain,
            hard_cases: meta.hard_cases,
            origin: Origin {
                generator: meta.generator.map(str::to_owned),
                run_id: None,
                group: meta.group,
            },
        },
    };
    if let Some((_, kind)) = check_record(&rec).into_iter().next() {
        let name = format!("{kind:?}");
        let short = name.split(['(', ' ', '{']).next().unwrap_or("bilinmeyen");
        return Err(format!("dogrulayici-{short}"));
    }
    Ok(rec)
}
