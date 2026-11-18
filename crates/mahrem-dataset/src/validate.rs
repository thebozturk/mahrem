//! Altın set ve tahmin dosyalarının doğrulanması.
//!
//! Hatalar kaydı kullanılamaz kılar. Uyarılar etiketleme yönergesine aykırı
//! görünen ama yanlış alarm olabilecek durumlardır; bir insan bakmalıdır.

use std::collections::{HashMap, HashSet};
use std::fmt;

use mahrem_core::{
    CodePointIndex, Label, is_valid_card_number, is_valid_iban_tr, is_valid_tckn, is_valid_vkn,
    is_valid_ykn, to_lowercase_tr,
};
use unicode_normalization::is_nfc;

use crate::prediction::PredictionRecord;
use crate::record::{GoldRecord, SCHEMA_VERSION};

/// Bulgunun ağırlığı.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Kayıt kullanılamaz.
    Error,
    /// Bir insan bakmalı.
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Severity::Error => "hata",
            Severity::Warning => "uyarı",
        })
    }
}

/// Bulgunun türü.
#[derive(Debug, Clone, PartialEq)]
pub enum IssueKind {
    /// Satır JSON olarak ya da şemaya göre okunamadı.
    Parse(String),
    /// Kimlik boş, çok uzun ya da izin verilmeyen karakter içeriyor.
    InvalidId,
    /// Kimlik daha önce görüldü.
    DuplicateId {
        /// İlk görüldüğü dosya.
        source: String,
        /// İlk görüldüğü satır.
        line: usize,
    },
    /// Desteklenmeyen şema sürümü.
    UnsupportedSchemaVersion(u32),
    /// Yönerge sürümü `X.Y.Z` biçiminde değil.
    InvalidGuidelineVersion(String),
    /// Belge metni boş.
    EmptyText,
    /// Belge metni NFC biçiminde değil.
    NotNfc,
    /// `start >= end`.
    EmptySpan,
    /// Aralık metnin dışına taşıyor.
    OutOfBounds {
        /// Metnin code point uzunluğu.
        len: usize,
    },
    /// Span'deki `text`, metindeki dilimle eşleşmiyor.
    TextMismatch {
        /// Metindeki dilim.
        actual: String,
    },
    /// Span'ler `start`'a göre sıralı değil.
    Unsorted,
    /// Span bir önceki span'le çakışıyor.
    Overlap {
        /// Çakıştığı span'in sırası.
        other: usize,
    },
    /// Span boşlukla başlıyor ya da bitiyor.
    SurroundingWhitespace,
    /// `child` işareti bu etikette kullanılamaz.
    ChildFlagNotAllowed(Label),
    /// Span kesme işaretli bir ekle bitiyor (`Ahmet'in`).
    SuffixInside,
    /// Span bir hitap içeriyor (`Ayşe Hanım`).
    HonorificInside(String),
    /// Span değeri çevreleyen bir tırnak, parantez ya da noktalamayla başlıyor ya da bitiyor.
    SurroundingPunctuation,
    /// Yapısal değer checksum doğrulamasından geçmiyor.
    ChecksumFailed,
    /// `TCKN` etiketli değer yabancı kimlik numarası gibi görünüyor.
    LooksLikeYkn,
    /// Etiketli bir değer belgenin başka bir yerinde etiketsiz geçiyor.
    UnlabeledRepeat {
        /// Etiketsiz geçişin başlangıcı.
        start: usize,
        /// Etiketsiz geçişin bitişi.
        end: usize,
    },
    /// Tahmin, altın sette olmayan bir belgeye ait.
    UnknownDocument,
    /// Skor sonlu değil ya da `[0, 1]` dışında.
    InvalidScore(f64),
    /// Altın setteki belge için tahmin yok.
    MissingPrediction,
}

impl IssueKind {
    /// Bulgunun ağırlığı.
    #[must_use]
    pub fn severity(&self) -> Severity {
        match self {
            IssueKind::SuffixInside
            | IssueKind::HonorificInside(_)
            | IssueKind::SurroundingPunctuation
            | IssueKind::ChecksumFailed
            | IssueKind::LooksLikeYkn
            | IssueKind::UnlabeledRepeat { .. }
            | IssueKind::MissingPrediction => Severity::Warning,
            _ => Severity::Error,
        }
    }
}

impl fmt::Display for IssueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueKind::Parse(msg) => write!(f, "okunamadı: {msg}"),
            IssueKind::InvalidId => f.write_str("geçersiz kimlik: [A-Za-z0-9._-], 1-128 karakter"),
            IssueKind::DuplicateId { source, line } => {
                write!(f, "kimlik tekrar ediyor; ilk geçtiği yer {source}:{line}")
            }
            IssueKind::UnsupportedSchemaVersion(v) => {
                write!(
                    f,
                    "desteklenmeyen şema sürümü {v}; beklenen {SCHEMA_VERSION}"
                )
            }
            IssueKind::InvalidGuidelineVersion(v) => {
                write!(f, "yönerge sürümü X.Y.Z biçiminde değil: {v:?}")
            }
            IssueKind::EmptyText => f.write_str("metin boş"),
            IssueKind::NotNfc => f.write_str("metin NFC biçiminde değil"),
            IssueKind::EmptySpan => f.write_str("start, end'den küçük olmalı"),
            IssueKind::OutOfBounds { len } => write!(f, "aralık metnin dışında (uzunluk {len})"),
            IssueKind::TextMismatch { actual } => {
                write!(
                    f,
                    "text alanı metinle eşleşmiyor; metindeki dilim {actual:?}"
                )
            }
            IssueKind::Unsorted => f.write_str("span'ler start'a göre sıralı değil"),
            IssueKind::Overlap { other } => write!(f, "{other}. span'le çakışıyor"),
            IssueKind::SurroundingWhitespace => f.write_str("span boşlukla başlıyor ya da bitiyor"),
            IssueKind::ChildFlagNotAllowed(l) => write!(f, "child işareti {l} için kullanılamaz"),
            IssueKind::SuffixInside => f.write_str("ek span'in içinde; ek dışarıda kalmalı"),
            IssueKind::HonorificInside(h) => {
                write!(f, "hitap span'in içinde ({h:?}); hitap dışarıda kalmalı")
            }
            IssueKind::SurroundingPunctuation => {
                f.write_str("span tırnak, parantez ya da noktalamayla başlıyor veya bitiyor")
            }
            IssueKind::ChecksumFailed => f.write_str("değer checksum doğrulamasından geçmiyor"),
            IssueKind::LooksLikeYkn => f.write_str("değer 99 ile başlıyor; etiket YKN olmalı"),
            IssueKind::UnlabeledRepeat { start, end } => {
                write!(f, "aynı değer {start}..{end} aralığında etiketsiz geçiyor")
            }
            IssueKind::UnknownDocument => f.write_str("bu kimlikte altın belge yok"),
            IssueKind::InvalidScore(s) => write!(f, "skor [0, 1] aralığında olmalı: {s}"),
            IssueKind::MissingPrediction => f.write_str("bu belge için tahmin yok"),
        }
    }
}

/// Bir bulgu ve yeri.
#[derive(Debug, Clone, PartialEq)]
pub struct Issue {
    /// Dosya adı ya da okuyanın verdiği kaynak adı.
    pub source: String,
    /// 1'den başlayan satır numarası; dosya düzeyindeki bulgularda 0.
    pub line: usize,
    /// Kayıt kimliği, okunabildiyse.
    pub id: Option<String>,
    /// Span'in kayıttaki sırası (0'dan başlar), bulgu bir span'e aitse.
    pub span: Option<usize>,
    /// Bulgunun türü.
    pub kind: IssueKind,
}

impl Issue {
    /// Bulgunun ağırlığı.
    #[must_use]
    pub fn severity(&self) -> Severity {
        self.kind.severity()
    }
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.source, self.line)?;
        if let Some(id) = &self.id {
            write!(f, " [{id}]")?;
        }
        if let Some(i) = self.span {
            write!(f, " span {i}")?;
        }
        write!(f, ": {}: {}", self.severity(), self.kind)
    }
}

/// Tek bir kaydın kendi içindeki kontrolleri. Kimlik benzersizliği gibi
/// dosyalar arası kontroller [`GoldSet`]'tedir.
///
/// Dönen çiftlerin ilk elemanı span sırasıdır; kayıt düzeyindeki bulgularda `None`.
#[must_use]
pub fn check_record(rec: &GoldRecord) -> Vec<(Option<usize>, IssueKind)> {
    let mut out = Vec::new();
    if !is_valid_id(&rec.id) {
        out.push((None, IssueKind::InvalidId));
    }
    if rec.meta.schema_version != SCHEMA_VERSION {
        out.push((
            None,
            IssueKind::UnsupportedSchemaVersion(rec.meta.schema_version),
        ));
    }
    if !is_valid_version(&rec.meta.guideline_version) {
        out.push((
            None,
            IssueKind::InvalidGuidelineVersion(rec.meta.guideline_version.clone()),
        ));
    }
    if rec.text.is_empty() {
        out.push((None, IssueKind::EmptyText));
    }
    if !is_nfc(&rec.text) {
        out.push((None, IssueKind::NotNfc));
    }

    let idx = CodePointIndex::new(&rec.text);
    // Aralığı geçerli olan span'ler: (sıra, start, end).
    let mut valid = Vec::new();
    for (i, span) in rec.spans.iter().enumerate() {
        let mut push = |kind| out.push((Some(i), kind));
        if span.child && !span.label.allows_child_flag() {
            push(IssueKind::ChildFlagNotAllowed(span.label));
        }
        if span.start >= span.end {
            push(IssueKind::EmptySpan);
            continue;
        }
        let Some(actual) = idx.slice(&rec.text, span.start..span.end) else {
            push(IssueKind::OutOfBounds { len: idx.len() });
            continue;
        };
        if actual != span.text {
            push(IssueKind::TextMismatch {
                actual: actual.to_owned(),
            });
        }
        if actual.starts_with(char::is_whitespace) || actual.ends_with(char::is_whitespace) {
            push(IssueKind::SurroundingWhitespace);
        }
        for kind in guideline_warnings(span.label, actual) {
            push(kind);
        }
        valid.push((i, span.start, span.end));
    }

    if rec.spans.windows(2).any(|w| w[0].start > w[1].start) {
        out.push((None, IssueKind::Unsorted));
    }
    let mut sorted = valid.clone();
    sorted.sort_by_key(|&(_, s, e)| (s, e));
    for w in sorted.windows(2) {
        let ((a, _, a_end), (b, b_start, _)) = (w[0], w[1]);
        if b_start < a_end {
            out.push((Some(b), IssueKind::Overlap { other: a }));
        }
    }

    out.extend(unlabeled_repeats(&rec.text, &idx, rec, &valid));
    out
}

fn is_valid_id(id: &str) -> bool {
    (1..=128).contains(&id.len())
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

fn is_valid_version(v: &str) -> bool {
    let parts: Vec<&str> = v.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

/// Etiketleme yönergesinde span dışında kalan hitap, unvan ve yakınlık
/// sözcükleri (`docs/gold-set/labeling-guideline.md`, bölüm 2).
const HONORIFICS: &[&str] = &[
    "bay",
    "bayan",
    "bey",
    "hanım",
    "hanim",
    "hanımefendi",
    "hanimefendi",
    "beyefendi",
    "sayın",
    "sayin",
    "sn",
    "sn.",
    "dr",
    "dr.",
    "av",
    "av.",
    "prof",
    "prof.",
    "doç",
    "doç.",
    "doc.",
    "uzm.",
    "op.",
    "abi",
    "abla",
    "teyze",
    "amca",
    "dayı",
    "dayi",
    "hala",
    "yenge",
    "hoca",
    "hocam",
    "öğretmen",
    "ogretmen",
];

/// Değeri çevreleyen ve span dışında kalması gereken işaretler. Nokta yok:
/// kısaltmalar ("Ltd. Şti.") değerin parçasıdır.
const OPENING: &[char] = &['"', '\'', '(', '[', '{', '“', '‘', '«'];
const CLOSING: &[char] = &[
    '"', '\'', ')', ']', '}', '”', '’', '»', ',', ';', ':', '!', '?',
];

/// Etiketleme yönergesine aykırı görünen durumlar.
fn guideline_warnings(label: Label, text: &str) -> Vec<IssueKind> {
    let mut out = Vec::new();
    if !matches!(label, Label::Email | Label::Secret) && has_trailing_suffix(text) {
        out.push(IssueKind::SuffixInside);
    }
    if label == Label::Person {
        let words: Vec<&str> = text.split_whitespace().collect();
        let edges = [words.first(), words.last()];
        if let Some(h) = edges
            .into_iter()
            .flatten()
            .find(|w| HONORIFICS.contains(&to_lowercase_tr(w).as_str()))
        {
            out.push(IssueKind::HonorificInside((*h).to_owned()));
        }
    }
    // Alan koduyla başlayan telefon numarası parantezle başlar: "(0212) 000 00 00".
    let phone_area_code = label == Label::Phone && text.starts_with('(');
    if (text.starts_with(OPENING) && !phone_area_code) || text.ends_with(CLOSING) {
        out.push(IssueKind::SurroundingPunctuation);
    }
    out.extend(checksum_warning(label, text));
    out
}

/// `Ahmet'in`, `Ankara’da` gibi kesme işaretli ekle mi bitiyor?
fn has_trailing_suffix(text: &str) -> bool {
    text.rfind(['\'', '’']).is_some_and(|pos| {
        let rest = &text[pos + 1..];
        pos > 0 && !rest.is_empty() && rest.chars().all(char::is_alphabetic)
    })
}

/// Yapısal değerleri ayraçlarından arındırıp çekirdekteki doğrulayıcıdan
/// geçirir. Yazıyla yazılmış numaralar gibi rakama indirgenemeyen değerler
/// atlanır.
fn checksum_warning(label: Label, text: &str) -> Option<IssueKind> {
    let compact: String = text
        .chars()
        .filter(|c| !c.is_whitespace() && !matches!(c, '-' | '.' | '/'))
        .collect();
    let digits = !compact.is_empty() && compact.bytes().all(|b| b.is_ascii_digit());
    let valid = match label {
        Label::Tckn if digits => {
            if is_valid_ykn(&compact) {
                return Some(IssueKind::LooksLikeYkn);
            }
            is_valid_tckn(&compact)
        }
        Label::Ykn if digits => is_valid_ykn(&compact),
        Label::Vkn if digits => is_valid_vkn(&compact),
        Label::CardNumber if digits => is_valid_card_number(&compact),
        Label::Iban => {
            let upper = compact.to_ascii_uppercase();
            if !upper.starts_with("TR") {
                return None;
            }
            is_valid_iban_tr(&upper)
        }
        _ => return None,
    };
    (!valid).then_some(IssueKind::ChecksumFailed)
}

/// Etiketli bir değerin belgenin başka bir yerinde etiketsiz geçmesi.
/// Yalnızca sözcük sınırında başlayan ve biten geçişler sayılır; ardından
/// gelen kesme işareti sınırdır.
fn unlabeled_repeats(
    text: &str,
    idx: &CodePointIndex,
    rec: &GoldRecord,
    valid: &[(usize, usize, usize)],
) -> Vec<(Option<usize>, IssueKind)> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let covered = |s: usize, e: usize| valid.iter().any(|&(_, vs, ve)| s < ve && vs < e);
    for &(i, _, _) in valid {
        let value = rec.spans[i].text.as_str();
        if value.chars().count() < 2 || !seen.insert(value) {
            continue;
        }
        for (byte, _) in text.match_indices(value) {
            let (Some(s), Some(e)) = (idx.char_of_byte(byte), idx.char_of_byte(byte + value.len()))
            else {
                continue;
            };
            let before = text[..byte].chars().next_back();
            let after = text[byte + value.len()..].chars().next();
            let boundary = before.is_none_or(|c| !c.is_alphanumeric())
                && after.is_none_or(|c| !c.is_alphanumeric());
            if boundary && !covered(s, e) {
                out.push((Some(i), IssueKind::UnlabeledRepeat { start: s, end: e }));
            }
        }
    }
    out
}

#[derive(Debug, Clone)]
struct DocInfo {
    source: String,
    line: usize,
    len: usize,
    text: String,
}

/// Birlikte doğrulanan altın set dosyaları. Kimlik benzersizliği eklenen
/// tüm dosyalar üzerinde aranır; tahminler bu belgelere karşı doğrulanır.
#[derive(Debug, Default)]
pub struct GoldSet {
    docs: HashMap<String, DocInfo>,
    order: Vec<String>,
    issues: Vec<Issue>,
}

impl GoldSet {
    /// Boş küme.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Bir JSONL dosyasının içeriğini ekler. Boş satırlar atlanır.
    pub fn add_jsonl(&mut self, source: &str, content: &str) {
        for (line, raw) in numbered_lines(content) {
            match serde_json::from_str::<GoldRecord>(raw) {
                Ok(rec) => self.add_record(source, line, &rec),
                Err(e) => self.issues.push(Issue {
                    source: source.to_owned(),
                    line,
                    id: None,
                    span: None,
                    kind: IssueKind::Parse(e.to_string()),
                }),
            }
        }
    }

    /// Okunmuş bir kaydı ekler.
    pub fn add_record(&mut self, source: &str, line: usize, rec: &GoldRecord) {
        let issue = |span, kind| Issue {
            source: source.to_owned(),
            line,
            id: Some(rec.id.clone()),
            span,
            kind,
        };
        if let Some(first) = self.docs.get(&rec.id) {
            let kind = IssueKind::DuplicateId {
                source: first.source.clone(),
                line: first.line,
            };
            self.issues.push(issue(None, kind));
        } else {
            self.order.push(rec.id.clone());
            self.docs.insert(
                rec.id.clone(),
                DocInfo {
                    source: source.to_owned(),
                    line,
                    len: rec.text.chars().count(),
                    text: rec.text.clone(),
                },
            );
        }
        for (span, kind) in check_record(rec) {
            self.issues.push(issue(span, kind));
        }
    }

    /// Eklenen belge sayısı (tekrar eden kimlikler bir kez sayılır).
    #[must_use]
    pub fn len(&self) -> usize {
        self.docs.len()
    }

    /// Hiç belge eklenmedi mi?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.docs.is_empty()
    }

    /// Şimdiye kadarki bulgular.
    #[must_use]
    pub fn issues(&self) -> &[Issue] {
        &self.issues
    }

    /// En az bir hata var mı?
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.issues.iter().any(|i| i.severity() == Severity::Error)
    }

    /// Bir tahmin dosyasını bu altın sete karşı doğrular.
    #[must_use]
    pub fn check_predictions(&self, source: &str, content: &str) -> Vec<Issue> {
        let mut out = Vec::new();
        let mut seen: HashMap<String, usize> = HashMap::new();
        for (line, raw) in numbered_lines(content) {
            let mut issue = |id: Option<&str>, span, kind| {
                out.push(Issue {
                    source: source.to_owned(),
                    line,
                    id: id.map(str::to_owned),
                    span,
                    kind,
                });
            };
            let rec = match serde_json::from_str::<PredictionRecord>(raw) {
                Ok(rec) => rec,
                Err(e) => {
                    issue(None, None, IssueKind::Parse(e.to_string()));
                    continue;
                }
            };
            let id = Some(rec.id.as_str());
            if let Some(&first) = seen.get(&rec.id) {
                let kind = IssueKind::DuplicateId {
                    source: source.to_owned(),
                    line: first,
                };
                issue(id, None, kind);
                continue;
            }
            seen.insert(rec.id.clone(), line);
            let Some(doc) = self.docs.get(&rec.id) else {
                issue(id, None, IssueKind::UnknownDocument);
                continue;
            };
            let cp = CodePointIndex::new(&doc.text);
            for (i, span) in rec.spans.iter().enumerate() {
                if span.start >= span.end {
                    issue(id, Some(i), IssueKind::EmptySpan);
                    continue;
                }
                let Some(actual) = cp.slice(&doc.text, span.start..span.end) else {
                    issue(id, Some(i), IssueKind::OutOfBounds { len: doc.len });
                    continue;
                };
                if span.text.as_deref().is_some_and(|t| t != actual) {
                    let kind = IssueKind::TextMismatch {
                        actual: actual.to_owned(),
                    };
                    issue(id, Some(i), kind);
                }
                if let Some(s) = span.score.filter(|s| !(0.0..=1.0).contains(s)) {
                    issue(id, Some(i), IssueKind::InvalidScore(s));
                }
            }
        }
        for id in self.order.iter().filter(|id| !seen.contains_key(*id)) {
            out.push(Issue {
                source: source.to_owned(),
                line: 0,
                id: Some(id.clone()),
                span: None,
                kind: IssueKind::MissingPrediction,
            });
        }
        out
    }
}

fn numbered_lines(content: &str) -> impl Iterator<Item = (usize, &str)> {
    content
        .lines()
        .enumerate()
        .map(|(i, l)| (i + 1, l))
        .filter(|(_, l)| !l.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[allow(clippy::needless_pass_by_value)]
    fn record(text: &str, spans: serde_json::Value) -> String {
        json!({
            "id": "t-1",
            "text": text,
            "spans": spans,
            "meta": {
                "schema_version": 1,
                "guideline_version": "0.1.0",
                "source": "handwritten",
                "domain": "chat",
                "hard_cases": [],
                "origin": {"generator": null, "run_id": null}
            }
        })
        .to_string()
    }

    fn span(start: usize, end: usize, label: &str, text: &str) -> serde_json::Value {
        json!({"start": start, "end": end, "label": label, "text": text,
               "uncertain": false, "child": false})
    }

    fn kinds(line: &str) -> Vec<IssueKind> {
        let mut set = GoldSet::new();
        set.add_jsonl("t", line);
        set.issues().iter().map(|i| i.kind.clone()).collect()
    }

    #[test]
    fn clean_record() {
        let text = "Ayşe'nin numarası 0555 000 00 00.";
        let line = record(
            text,
            json!([
                span(0, 4, "PERSON", "Ayşe"),
                span(18, 32, "PHONE", "0555 000 00 00")
            ]),
        );
        assert_eq!(kinds(&line), vec![]);
    }

    #[test]
    fn empty_spans_are_fine() {
        assert_eq!(kinds(&record("Kargom nerede?", json!([]))), vec![]);
    }

    #[test]
    fn code_point_offsets() {
        // "İ" iki bayt, "🙂" dört bayt; konumlar code point cinsinden.
        let line = record("🙂 İrem geldi", json!([span(2, 6, "PERSON", "İrem")]));
        assert_eq!(kinds(&line), vec![]);
    }

    #[test]
    fn unknown_label_and_field() {
        let line = record("x", json!([span(0, 1, "NAME", "x")]));
        assert!(matches!(kinds(&line)[..], [IssueKind::Parse(_)]));
        let line = line.replace("\"uncertain\"", "\"unsure\"");
        assert!(matches!(kinds(&line)[..], [IssueKind::Parse(_)]));
    }

    #[test]
    fn not_nfc() {
        // "i" + birleşen nokta değil, "e" + birleşen akut: NFC'de tek karaktere iner.
        let line = record("Mehme\u{0301}t", json!([]));
        assert_eq!(kinds(&line), vec![IssueKind::NotNfc]);
    }

    #[test]
    fn span_errors() {
        let text = "Ali ve Veli";
        assert_eq!(
            kinds(&record(text, json!([span(3, 3, "PERSON", "")]))),
            vec![IssueKind::EmptySpan]
        );
        assert_eq!(
            kinds(&record(text, json!([span(7, 12, "PERSON", "Veli")]))),
            vec![IssueKind::OutOfBounds { len: 11 }]
        );
        assert_eq!(
            kinds(&record(text, json!([span(0, 3, "PERSON", "Ayşe")]))),
            vec![IssueKind::TextMismatch {
                actual: "Ali".into()
            }]
        );
        assert_eq!(
            kinds(&record(text, json!([span(0, 4, "PERSON", "Ali ")]))),
            vec![IssueKind::SurroundingWhitespace]
        );
    }

    #[test]
    fn order_and_overlap() {
        let text = "Ali ve Veli";
        let unsorted = record(
            text,
            json!([span(7, 11, "PERSON", "Veli"), span(0, 3, "PERSON", "Ali")]),
        );
        assert_eq!(kinds(&unsorted), vec![IssueKind::Unsorted]);
        let overlap = record(
            text,
            json!([
                span(0, 6, "PERSON", "Ali ve"),
                span(4, 11, "PERSON", "ve Veli")
            ]),
        );
        assert_eq!(kinds(&overlap), vec![IssueKind::Overlap { other: 0 }]);
    }

    #[test]
    fn child_flag() {
        let mut s = span(0, 4, "PHONE", "0555");
        s["child"] = json!(true);
        let line = record("0555", json!([s]));
        assert_eq!(
            kinds(&line),
            vec![IssueKind::ChildFlagNotAllowed(Label::Phone)]
        );
    }

    #[test]
    fn suffix_and_honorific_warnings() {
        let line = record("Ahmet'in", json!([span(0, 8, "PERSON", "Ahmet'in")]));
        assert_eq!(kinds(&line), vec![IssueKind::SuffixInside]);
        let line = record("Ayşe Hanım", json!([span(0, 10, "PERSON", "Ayşe Hanım")]));
        assert_eq!(
            kinds(&line),
            vec![IssueKind::HonorificInside("Hanım".into())]
        );
        let line = record("Dr. Kaya", json!([span(0, 8, "PERSON", "Dr. Kaya")]));
        assert_eq!(kinds(&line), vec![IssueKind::HonorificInside("Dr.".into())]);
        let line = record("Fatma Teyze", json!([span(0, 11, "PERSON", "Fatma Teyze")]));
        assert_eq!(
            kinds(&line),
            vec![IssueKind::HonorificInside("Teyze".into())]
        );
    }

    #[test]
    fn punctuation_warning() {
        let line = record("\"Ayşe\" dedi", json!([span(0, 6, "PERSON", "\"Ayşe\"")]));
        assert_eq!(kinds(&line), vec![IssueKind::SurroundingPunctuation]);
        let line = record("Ali, Veli", json!([span(0, 4, "PERSON", "Ali,")]));
        assert_eq!(kinds(&line), vec![IssueKind::SurroundingPunctuation]);
        let line = record(
            "(0212) 000 00 00",
            json!([span(0, 16, "PHONE", "(0212) 000 00 00")]),
        );
        assert_eq!(kinds(&line), vec![]);
        // Kısaltma noktası değerin parçasıdır.
        let line = record(
            "Örnek Ltd. Şti.",
            json!([span(0, 15, "ORGANIZATION", "Örnek Ltd. Şti.")]),
        );
        assert_eq!(kinds(&line), vec![]);
    }

    #[test]
    fn checksum_warnings() {
        let ok = record("10000000146", json!([span(0, 11, "TCKN", "10000000146")]));
        assert_eq!(kinds(&ok), vec![]);
        let bad = record("10000000145", json!([span(0, 11, "TCKN", "10000000145")]));
        assert_eq!(kinds(&bad), vec![IssueKind::ChecksumFailed]);
        let ykn = record("99000000042", json!([span(0, 11, "TCKN", "99000000042")]));
        assert_eq!(kinds(&ykn), vec![IssueKind::LooksLikeYkn]);
        let spaced = "TR33 0006 1005 1978 6457 8413 26";
        let iban = record(spaced, json!([span(0, 32, "IBAN", spaced)]));
        assert_eq!(kinds(&iban), vec![]);
        let card = record(
            "4111-1111-1111-1112",
            json!([span(0, 19, "CARD_NUMBER", "4111-1111-1111-1112")]),
        );
        assert_eq!(kinds(&card), vec![IssueKind::ChecksumFailed]);
        // Yazıyla yazılmış numara doğrulanamaz; uyarı verilmez.
        let spelled = record("bir sıfır", json!([span(0, 9, "TCKN", "bir sıfır")]));
        assert_eq!(kinds(&spelled), vec![]);
    }

    #[test]
    fn unlabeled_repeat() {
        let text = "Elif geldi, Elif'e söyle. Elifçe değil.";
        let line = record(text, json!([span(0, 4, "PERSON", "Elif")]));
        assert_eq!(
            kinds(&line),
            vec![IssueKind::UnlabeledRepeat { start: 12, end: 16 }]
        );
    }

    #[test]
    fn duplicate_ids_across_files() {
        let mut set = GoldSet::new();
        let line = record("x", json!([]));
        set.add_jsonl("a.jsonl", &line);
        set.add_jsonl("b.jsonl", &format!("\n{line}"));
        let issues = set.issues();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].source, "b.jsonl");
        assert_eq!(issues[0].line, 2);
        assert_eq!(
            issues[0].kind,
            IssueKind::DuplicateId {
                source: "a.jsonl".into(),
                line: 1
            }
        );
        assert!(set.has_errors());
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn record_level_errors() {
        let line = record("x", json!([]))
            .replace("\"t-1\"", "\"t 1\"")
            .replace("\"0.1.0\"", "\"v1\"")
            .replace("\"schema_version\":1", "\"schema_version\":2");
        assert_eq!(
            kinds(&line),
            vec![
                IssueKind::InvalidId,
                IssueKind::UnsupportedSchemaVersion(2),
                IssueKind::InvalidGuidelineVersion("v1".into()),
            ]
        );
    }

    #[test]
    fn predictions() {
        let mut set = GoldSet::new();
        set.add_jsonl(
            "gold",
            &record("Ali ve Veli", json!([span(0, 3, "PERSON", "Ali")])),
        );
        let preds = [
            r#"{"id":"t-1","spans":[{"start":0,"end":3,"label":"PERSON","score":0.9},"#,
            r#"{"start":0,"end":6,"label":"PERSON","text":"Ali"},"#,
            r#"{"start":7,"end":20,"label":"PERSON"},{"start":0,"end":1,"label":"AGE","score":1.5}]}"#,
            "\n",
            r#"{"id":"nope","spans":[]}"#,
        ]
        .concat();
        let got: Vec<IssueKind> = set
            .check_predictions("p", &preds)
            .into_iter()
            .map(|i| i.kind)
            .collect();
        assert_eq!(
            got,
            vec![
                IssueKind::TextMismatch {
                    actual: "Ali ve".into()
                },
                IssueKind::OutOfBounds { len: 11 },
                IssueKind::InvalidScore(1.5),
                IssueKind::UnknownDocument,
            ]
        );
        let missing = set.check_predictions("p", "");
        assert_eq!(missing[0].kind, IssueKind::MissingPrediction);
        assert_eq!(missing[0].severity(), Severity::Warning);
    }
}
