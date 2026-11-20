//! Mahrem'in Python bağlaması. Python API'si `mahrem` paketindedir; bu modül
//! (`mahrem._mahrem`) yalnızca Rust çekirdeğini açar.

use pyo3::prelude::*;

/// T.C. kimlik numarasını doğrular.
#[pyfunction]
fn is_valid_tckn(s: &str) -> bool {
    mahrem_core::is_valid_tckn(s)
}

/// Yabancı kimlik numarasını doğrular.
#[pyfunction]
fn is_valid_ykn(s: &str) -> bool {
    mahrem_core::is_valid_ykn(s)
}

/// Vergi kimlik numarasını doğrular.
#[pyfunction]
fn is_valid_vkn(s: &str) -> bool {
    mahrem_core::is_valid_vkn(s)
}

/// Türkiye IBAN'ını doğrular.
#[pyfunction]
fn is_valid_iban_tr(s: &str) -> bool {
    mahrem_core::is_valid_iban_tr(s)
}

/// Kart numarasını Luhn ile doğrular.
#[pyfunction]
fn is_valid_card_number(s: &str) -> bool {
    mahrem_core::is_valid_card_number(s)
}

/// Metni Türkçe kurallarla küçük harfe çevirir.
#[pyfunction]
fn to_lowercase_tr(s: &str) -> String {
    mahrem_core::to_lowercase_tr(s)
}

/// Metni Türkçe kurallarla büyük harfe çevirir.
#[pyfunction]
fn to_uppercase_tr(s: &str) -> String {
    mahrem_core::to_uppercase_tr(s)
}

/// Bulunan bir varlık.
#[pyclass(frozen, get_all, module = "mahrem")]
struct Entity {
    /// Etiket kimliği (`TCKN`, `PHONE` …).
    label: String,
    /// Başlangıç (code point, dahil).
    start: usize,
    /// Bitiş (code point, hariç).
    end: usize,
    /// Metindeki karşılığı.
    text: String,
    /// Güven skoru, `[0, 1]`.
    score: f32,
    /// Tanıyıcının adı.
    source: String,
}

#[pymethods]
impl Entity {
    fn __repr__(&self) -> String {
        format!(
            "Entity(label={:?}, start={}, end={}, text={:?}, score={:.2}, source={:?})",
            self.label, self.start, self.end, self.text, self.score, self.source
        )
    }
}

/// Metindeki yapısal kişisel verileri bulur: TCKN/YKN, VKN, IBAN, kart,
/// telefon ve e-posta. Konumlar Python `str` indeksleridir.
#[pyfunction]
#[pyo3(signature = (text, min_score = mahrem_core::DEFAULT_MIN_SCORE))]
fn analyze(text: &str, min_score: f32) -> Vec<Entity> {
    mahrem_core::analyze_with(text, min_score)
        .iter()
        .map(|e| Entity {
            label: e.label.id().to_owned(),
            start: e.start,
            end: e.end,
            text: e.text(text).to_owned(),
            score: e.score,
            source: e.source.to_owned(),
        })
        .collect()
}

/// `mahrem._mahrem` modülü.
#[pymodule(gil_used = false)]
fn _mahrem(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("DEFAULT_MIN_SCORE", mahrem_core::DEFAULT_MIN_SCORE)?;
    m.add_class::<Entity>()?;
    m.add_function(wrap_pyfunction!(analyze, m)?)?;
    m.add_function(wrap_pyfunction!(is_valid_tckn, m)?)?;
    m.add_function(wrap_pyfunction!(is_valid_ykn, m)?)?;
    m.add_function(wrap_pyfunction!(is_valid_vkn, m)?)?;
    m.add_function(wrap_pyfunction!(is_valid_iban_tr, m)?)?;
    m.add_function(wrap_pyfunction!(is_valid_card_number, m)?)?;
    m.add_function(wrap_pyfunction!(to_lowercase_tr, m)?)?;
    m.add_function(wrap_pyfunction!(to_uppercase_tr, m)?)?;
    Ok(())
}
