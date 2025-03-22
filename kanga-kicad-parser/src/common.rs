use serde::{de::Deserializer, ser::Serializer, Deserialize};

mod color;
mod fill;
mod font;
mod line_style;
mod offset;
mod paper;
mod points;
mod position;
mod property;
mod size;
mod stroke;
mod symbol;
mod text_effects;
mod text_justify;
mod title_block;

pub use {
    color::*, fill::*, font::*, line_style::*, offset::*, paper::*, points::*, position::*, property::*, size::*,
    stroke::*, symbol::*, text_effects::*, text_justify::*, title_block::*,
};

/// Convert from millimeters to nanometers.
pub fn deserialize_mm_to_nm<'de, D>(d: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let v: f64 = Deserialize::deserialize(d)?;
    Ok((v * 1e6) as i64)
}

/// Convert from millimeters to nanometers, unsigned.
pub fn deserialize_mm_to_unsigned_nm<'de, D>(d: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let v: f64 = Deserialize::deserialize(d)?;
    if v < 0.0 {
        return Err(serde::de::Error::custom("negative value"));
    }
    Ok((v * 1e6) as u64)
}

/// Convert from millimeters to nanometers, wrapping in an `Option<i64>` type.
pub fn deserialize_mm_to_opt_nm<'de, D>(d: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let v: Option<f64> = Deserialize::deserialize(d)?;
    Ok(v.map(|v| (v * 1e6) as i64))
}

/// Convert from nanometers to millimeters
pub fn serialize_nm_to_mm<S>(v: &i64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_f64((*v as f64) * 1e-6)
}

/// Convert from unsigned nanometers to millimeters.
pub fn serialize_unsigned_nm_to_mm<S>(v: &u64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_f64((*v as f64) * 1e-6)
}

/// Convert from nanometers to millimeters if the value is `Some<i64>`.
pub fn serialize_opt_nm_to_mm<S>(v: &Option<i64>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(v) = v {
        s.serialize_f64((*v as f64) * 1e-6)
    } else {
        s.serialize_none()
    }
}
