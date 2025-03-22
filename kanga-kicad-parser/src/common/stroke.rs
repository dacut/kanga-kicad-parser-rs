use {
    super::{deserialize_mm_to_opt_nm, serialize_opt_nm_to_mm, Color, LineStyle},
    crate::{impl_try_from_cons_value, LexprExt, ParseError},
    lexpr::Cons,
    serde::{Deserialize, Serialize},
};

/// KiCad stroke definition.
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-intro/index.html#_stroke_definition)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "stroke")]
pub struct Stroke {
    /// Width in nanometers.
    ///
    /// KiCad serializes in millimeters but uses nanometers internally.
    #[serde(deserialize_with = "deserialize_mm_to_opt_nm", serialize_with = "serialize_opt_nm_to_mm")]
    pub width: Option<i64>,

    /// Line style.
    #[serde(rename = "type")]
    pub line_style: Option<LineStyle>,

    /// Color in RGBA format.
    pub color: Option<Color>,
}

impl TryFrom<&Cons> for Stroke {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, Self::Error> {
        let mut width = None;
        let mut line_style = None;
        let mut color = None;

        let mut rest = cons.expect_cons_with_symbol_head("stroke")?;
        while !rest.is_null() {
            let r_cons = rest.expect_cons()?;
            let element = r_cons.car();
            rest = r_cons.cdr();
            let (key, cdr) = element.expect_cons_with_any_symbol_head()?;

            match key {
                "width" => {
                    let (value, cdr) = cdr.expect_cons_with_any_float_head()?;
                    cdr.expect_null()?;
                    width = Some((value * 1e6) as i64);
                }

                "type" => {
                    line_style = Some(LineStyle::try_from(element)?);
                }

                "color" => {
                    color = Some(Color::try_from(element)?);
                }

                _ => {
                    return Err(ParseError::Unexpected(element.clone()));
                }
            }
        }

        Ok(Self {
            width,
            line_style,
            color,
        })
    }
}

impl_try_from_cons_value!(Stroke);
