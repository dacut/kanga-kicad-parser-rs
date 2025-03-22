use {
    super::{deserialize_mm_to_opt_nm, serialize_opt_nm_to_mm, Size},
    crate::{impl_try_from_cons_value, LexprExt, ParseError},
    lexpr::{Cons, Value},
    serde::{Deserialize, Serialize},
};

/// KiCad font for text effects.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "font")]
pub struct Font {
    /// Font family name. Either a TrueType font family name or `"KiCad Font"` for the KiCad stroke
    /// font.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub face: Option<String>,

    /// The size; this is specified as a height x width pair.
    pub size: Size,

    /// Thickness in nanometers.
    ///
    /// KiCad serializes in millimeters but uses nanometers internally.
    #[serde(deserialize_with = "deserialize_mm_to_opt_nm", serialize_with = "serialize_opt_nm_to_mm")]
    pub thickness: Option<i64>,

    /// Whether the font is in boldface type.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub bold: bool,

    /// Whether the font is in italic type.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub italic: bool,

    /// Line spacing in nanometers.
    #[serde(
        deserialize_with = "deserialize_mm_to_opt_nm",
        serialize_with = "serialize_opt_nm_to_mm",
        skip_serializing_if = "Option::is_none"
    )]
    pub line_spacing: Option<i64>,
}

impl TryFrom<&Cons> for Font {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, Self::Error> {
        let mut rest = cons.expect_cons_with_symbol_head("font")?;
        let mut face = None;
        let mut size = None;
        let mut thickness = None;
        let mut bold = false;
        let mut italic = false;
        let mut line_spacing = None;

        while !rest.is_null() {
            let r_cons = rest.expect_cons()?;
            let element = r_cons.car();
            rest = r_cons.cdr();

            if let Some(e_cons) = element.as_cons() {
                let (key, cdr) = e_cons.expect_cons_with_any_symbol_head()?;

                match key {
                    "face" => {
                        let (value, cdr) = cdr.expect_cons_with_any_str_head()?;
                        cdr.expect_null()?;
                        face = Some(value.to_string());
                    }

                    "size" => {
                        size = Some(Size::try_from(e_cons)?);
                    }

                    "thickness" => {
                        let (value, cdr) = cdr.expect_cons_with_any_float_head()?;
                        cdr.expect_null()?;
                        thickness = Some((value * 1e6) as i64);
                    }

                    "line_spacing" => {
                        let (value, cdr) = cdr.expect_cons_with_any_float_head()?;
                        cdr.expect_null()?;
                        line_spacing = Some((value * 1e6) as i64);
                    }

                    _ => return Err(ParseError::Unexpected(element.clone())),
                }
            } else if let Some(sym) = element.as_symbol() {
                match sym {
                    "bold" => bold = true,
                    "italic" => italic = true,
                    _ => return Err(ParseError::Unexpected(element.clone())),
                }
            } else {
                return Err(ParseError::Unexpected(element.clone()));
            }
        }

        let Some(size) = size else {
            return Err(ParseError::missing_field("font", "size", cons.clone()));
        };

        Ok(Self {
            face,
            size,
            thickness,
            bold,
            italic,
            line_spacing,
        })
    }
}

impl_try_from_cons_value!(Font);
