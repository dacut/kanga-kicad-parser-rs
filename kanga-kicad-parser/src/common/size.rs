use {
    super::{deserialize_mm_to_unsigned_nm, serialize_unsigned_nm_to_mm},
    crate::{impl_try_from_cons_value, LexprExt, ParseError},
    lexpr::{Cons, Value},
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "size")]
pub struct Size {
    /// Width in nanometers.
    ///
    /// KiCad serializes in millimeters but uses nanometers internally.
    #[serde(deserialize_with = "deserialize_mm_to_unsigned_nm", serialize_with = "serialize_unsigned_nm_to_mm")]
    pub width: u64,

    /// Height in nanometers.
    ///
    /// KiCad serializes in millimeters but uses nanometers internally.
    #[serde(deserialize_with = "deserialize_mm_to_unsigned_nm", serialize_with = "serialize_unsigned_nm_to_mm")]
    pub height: u64,
}

impl Size {
    /// Create a new `Size` object with the specified width and height in nm.
    pub fn new(width: u64, height: u64) -> Self {
        Self {
            width,
            height,
        }
    }

    /// Create a new `Size` object with the specified width and height in mm.
    pub fn from_mm(width: f64, height: f64) -> Result<Self, ParseError> {
        if width < 0.0 {
            return Err(ParseError::InvalidWidth(width));
        }

        if height < 0.0 {
            return Err(ParseError::InvalidHeight(height));
        }

        Ok(Self {
            width: (width * 1e6) as u64,
            height: (height * 1e6) as u64,
        })
    }

    /// Attempt to create a Size value from a (height width) cons cell.
    pub(crate) fn try_from_hw_cons(rest: &Value) -> Result<Self, ParseError> {
        // KiCad doesn't follow normal conventions and places height first.
        let (height, rest) = rest.expect_cons_with_any_float_head()?;
        let (width, rest) = rest.expect_cons_with_any_float_head()?;
        rest.expect_null()?;

        Self::from_mm(width, height)
    }
}

impl TryFrom<&Cons> for Size {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, Self::Error> {
        let rest = cons.expect_cons_with_symbol_head("size")?;
        Size::try_from_hw_cons(rest)
    }
}

impl_try_from_cons_value!(Size);
