use {
    crate::{impl_try_from_cons_value, LexprExt, ParseError},
    lexpr::Cons,
    serde::{Deserialize, Serialize},
};

/// KiCad text justification definition.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TextJustify {
    /// Horizontal justification.
    #[serde(skip_serializing_if = "HorizJustify::is_default")]
    pub horiz_justify: HorizJustify,

    /// Vertical justification.
    #[serde(skip_serializing_if = "VertJustify::is_default")]
    pub vert_justify: VertJustify,

    /// Whether the text is mirrored.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub mirror: bool,
}

/// Horizontal justification.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HorizJustify {
    /// Left justified.
    Left,

    /// Centered.
    #[default]
    Center,

    /// Right justified.
    Right,
}

/// Vertical justification.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VertJustify {
    /// Top justified.
    Top,

    /// Centered.
    #[default]
    Center,

    /// Bottom justified.
    Bottom,
}

impl HorizJustify {
    /// Indicates whether this is the default justification.
    #[inline(always)]
    pub fn is_default(&self) -> bool {
        matches!(self, HorizJustify::Center)
    }
}

impl VertJustify {
    /// Indicates whether this is the default justification.
    #[inline(always)]
    pub fn is_default(&self) -> bool {
        matches!(self, VertJustify::Center)
    }
}

impl TryFrom<&Cons> for TextJustify {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, Self::Error> {
        let mut rest = cons.expect_cons_with_symbol_head("justify")?;
        let mut horiz_justify = HorizJustify::Center;
        let mut vert_justify = VertJustify::Center;
        let mut mirror = false;

        while !rest.is_null() {
            let cons = rest.expect_cons()?;
            let car = cons.car();
            rest = cons.cdr();

            let Some(key) = car.as_symbol() else {
                return Err(ParseError::Unexpected(car.clone()));
            };

            match key {
                "left" => horiz_justify = HorizJustify::Left,
                "right" => horiz_justify = HorizJustify::Right,
                "top" => vert_justify = VertJustify::Top,
                "bottom" => vert_justify = VertJustify::Bottom,
                "mirror" => mirror = true,
                _ => return Err(ParseError::Unexpected(car.clone())),
            }
        }

        Ok(Self {
            horiz_justify,
            vert_justify,
            mirror,
        })
    }
}

impl_try_from_cons_value!(TextJustify);
