use {
    crate::{impl_try_from_cons_value, LexprExt, ParseError},
    lexpr::{Cons, Value},
    serde::{Deserialize, Serialize},
};

/// KiCad schematic and symbol graphical fill definition.
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-intro/index.html#_symbol_arc)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "fill")]
pub struct Fill {
    /// Fill type.
    #[serde(rename = "type")]
    pub fill_type: FillType,
}

/// KiCad fill type.
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-intro/index.html#_symbol_arc)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FillType {
    /// No fill.
    None,

    /// Filled with the outline color.
    Outline,

    /// Filled with the theme background color.
    Background,
}

impl TryFrom<&Cons> for Fill {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, Self::Error> {
        let mut fill_type = FillType::None;

        let mut rest = cons.expect_cons_with_symbol_head("fill")?;
        while !rest.is_null() {
            let r_cons = rest.expect_cons()?;
            let element = r_cons.car();
            rest = r_cons.cdr();
            let (key, _) = element.expect_cons_with_any_symbol_head()?;

            match key {
                "type" => {
                    fill_type = FillType::try_from(element)?;
                }

                _ => {
                    return Err(ParseError::Unexpected(element.clone()));
                }
            }
        }

        Ok(Self {
            fill_type,
        })
    }
}

impl_try_from_cons_value!(Fill);

impl TryFrom<&Cons> for FillType {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, Self::Error> {
        let rest = cons.expect_cons_with_symbol_head("type")?;
        let (value, rest) = rest.expect_cons_with_any_symbol_head()?;
        rest.expect_null()?;

        match value {
            "none" => Ok(FillType::None),
            "outline" => Ok(FillType::Outline),
            "background" => Ok(FillType::Background),
            _ => Err(ParseError::Unexpected(Value::Cons(cons.clone()))),
        }
    }
}

impl_try_from_cons_value!(FillType);
