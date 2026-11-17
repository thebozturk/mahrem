//! [`Label`]'ı kimliğiyle (`"PERSON"`) okuyup yazar. Çekirdek serde'ye
//! bağımlı olmasın diye bu crate'te durur.

use mahrem_core::Label;
use serde::{Deserialize, Deserializer, Serializer, de::Error as _};

// serde `with` imzası referans ister.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub(crate) fn serialize<S: Serializer>(label: &Label, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(label.id())
}

pub(crate) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Label, D::Error> {
    let id = <std::borrow::Cow<'de, str>>::deserialize(d)?;
    id.parse().map_err(D::Error::custom)
}
