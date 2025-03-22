use {
    super::{deserialize_mm_to_nm, serialize_nm_to_mm},
    serde::{Deserialize, Serialize},
};

/// KiCad offset.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "offset")]
pub struct Offset {
    /// X offset in nanometers.
    ///
    /// KiCad serializes in millimeters but uses nanometers internally.
    #[serde(deserialize_with = "deserialize_mm_to_nm", serialize_with = "serialize_nm_to_mm")]
    pub x: i64,

    /// Y offset in nanometers.
    ///
    /// KiCad serializes in millimeters but uses nanometers internally.
    #[serde(deserialize_with = "deserialize_mm_to_nm", serialize_with = "serialize_nm_to_mm")]
    pub y: i64,
}
