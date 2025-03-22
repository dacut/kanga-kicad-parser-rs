use {
    super::{deserialize_mm_to_nm, serialize_nm_to_mm},
    crate::{impl_try_from_cons_value, LexprExt, ParseError},
    lexpr::{Cons, Value},
    serde::{Deserialize, Serialize},
};

/// KiCad position identifier.
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-intro/index.html#_position_identifier)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "at")]
pub struct Position {
    /// X coordinate in nanometers.
    ///
    /// KiCad serializes in millimeters but uses nanometers internally.
    #[serde(deserialize_with = "deserialize_mm_to_nm", serialize_with = "serialize_nm_to_mm")]
    pub x: i64,

    /// Y coordinate in nanometers.
    ///
    /// KiCad serializes in millimeters but uses nanometers internally.
    #[serde(deserialize_with = "deserialize_mm_to_nm", serialize_with = "serialize_nm_to_mm")]
    pub y: i64,

    /// Angle in degrees.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub angle: Option<f64>,
}

impl Position {
    pub(crate) fn try_from_xy_cons(cons: &Value) -> Result<Self, ParseError> {
        let (x, rest) = cons.expect_cons_with_any_float_head()?;
        let (y, rest) = rest.expect_cons_with_any_float_head()?;
        let mut angle = None;

        if !rest.is_null() {
            let (value, rest) = rest.expect_cons_with_any_float_head()?;
            rest.expect_null()?;
            angle = Some(value);
        } else {
            rest.expect_null()?;
        }

        let x = (x * 1e6) as i64;
        let y = (y * 1e6) as i64;

        Ok(Self {
            x,
            y,
            angle,
        })
    }
}

impl TryFrom<&Cons> for Position {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, Self::Error> {
        let rest = cons.expect_cons_with_symbol_head("at")?;
        Self::try_from_xy_cons(rest)
    }
}

impl_try_from_cons_value!(Position);
