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

/// `mahrem._mahrem` modülü.
#[pymodule(gil_used = false)]
fn _mahrem(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_function(wrap_pyfunction!(is_valid_tckn, m)?)?;
    m.add_function(wrap_pyfunction!(is_valid_ykn, m)?)?;
    m.add_function(wrap_pyfunction!(is_valid_vkn, m)?)?;
    m.add_function(wrap_pyfunction!(is_valid_iban_tr, m)?)?;
    m.add_function(wrap_pyfunction!(is_valid_card_number, m)?)?;
    m.add_function(wrap_pyfunction!(to_lowercase_tr, m)?)?;
    m.add_function(wrap_pyfunction!(to_uppercase_tr, m)?)?;
    Ok(())
}
