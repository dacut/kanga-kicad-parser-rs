use {
    crate::{impl_try_from_cons_value, LexprExt, ParseError},
    lexpr::{Cons, Value},
    serde::{Deserialize, Serialize},
};

/// KiCad stroke line styles.
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-intro/index.html#_stroke_definition)
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename = "type", rename_all = "snake_case")]
pub enum LineStyle {
    /// Dash
    Dash,

    /// Dash dot
    DashDot,

    /// Dash Dot Dot (version 7+)
    DashDotDot,

    /// Dot
    Dot,

    /// Default
    #[default]
    Default,

    /// Solid
    Solid,
}

impl TryFrom<&Cons> for LineStyle {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, Self::Error> {
        let rest = cons.expect_cons_with_symbol_head("type")?;
        let (value, rest) = rest.expect_cons_with_any_symbol_head()?;
        rest.expect_null()?;

        match value {
            "dash" => Ok(LineStyle::Dash),
            "dash_dot" => Ok(LineStyle::DashDot),
            "dash_dot_dot" => Ok(LineStyle::DashDotDot),
            "dot" => Ok(LineStyle::Dot),
            "default" => Ok(LineStyle::Default),
            "solid" => Ok(LineStyle::Solid),
            _ => Err(ParseError::Unexpected(Value::Cons(cons.clone()))),
        }
    }
}

impl_try_from_cons_value!(LineStyle);
