use {
    crate::{impl_try_from_cons_value, LexprExt, ParseError},
    lexpr::Cons,
    serde::{Deserialize, Serialize},
};

/// Color in RGBA format
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "color")]
pub struct Color {
    /// Red component
    pub red: f64,

    /// Green component
    pub green: f64,

    /// Blue component
    pub blue: f64,

    /// Alpha component
    pub alpha: Option<f64>,
}

impl TryFrom<&Cons> for Color {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, Self::Error> {
        let rest = cons.expect_cons_with_symbol_head("color")?;
        let (red, rest) = rest.expect_cons_with_any_float_head()?;
        let (green, rest) = rest.expect_cons_with_any_float_head()?;
        let (blue, rest) = rest.expect_cons_with_any_float_head()?;

        let alpha = if rest.is_null() {
            None
        } else {
            let (value, cdr) = rest.expect_cons_with_any_float_head()?;
            cdr.expect_null()?;
            Some(value)
        };

        Ok(Self {
            red,
            green,
            blue,
            alpha,
        })
    }
}

impl_try_from_cons_value!(Color);
