use {
    super::Size,
    crate::{impl_try_from_cons_value, LexprExt, ParseError},
    lexpr::Cons,
    serde::{Deserialize, Serialize},
};

/// KiCad page settings: page size and orientation.
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-intro/index.html#_page_settings)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "paper")]
pub struct Paper {
    /// Page size; either a standard size or custom.
    pub paper_size: PaperSize,

    /// Page orientation.
    #[serde(default, skip_serializing_if = "PaperOrientation::is_landscape")]
    pub orientation: PaperOrientation,
}

/// KiCad page orientation.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum PaperOrientation {
    /// Landscape orientation.
    #[default]
    Landscape,

    /// Portrait orientation.
    Portrait,
}

/// KiCad page size. This is either a standard ISO or ANSI size, or a custom size.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PaperSize {
    /// ISO A0: 841 x 1189 mm.
    #[serde(rename = "A0")]
    IsoA0,

    /// ISO A1: 594 x 841 mm.
    #[serde(rename = "A1")]
    IsoA1,

    /// ISO A2: 420 x 594 mm.
    #[serde(rename = "A2")]
    IsoA2,

    /// ISO A3: 297 x 420 mm.
    #[serde(rename = "A3")]
    IsoA3,

    /// ISO A4: 210 x 297 mm.
    #[serde(rename = "A4")]
    IsoA4,

    /// ISO A5: 148 x 210 mm.
    #[serde(rename = "A5")]
    IsoA5,

    /// ANSI A (Letter): 8.5 x 11 in, 216 x 279 mm.
    #[serde(rename = "A")]
    AnsiA,

    /// ANSI B (Tabloid): 11 x 17 in, 279 x 432 mm.
    #[serde(rename = "B")]
    AnsiB,

    /// ANSI C: 17 x 22 in, 432 x 559 mm.
    #[serde(rename = "C")]
    AnsiC,

    /// ANSI D: 22 x 34 in, 559 x 864 mm.
    #[serde(rename = "D")]
    AnsiD,

    /// ANSI E: 34 x 44 in, 864 x 1118 mm.
    #[serde(rename = "E")]
    AnsiE,

    /// Custom size.
    User(Size),
}

impl PaperOrientation {
    /// Indicates whether this is landscape orientation.
    #[inline(always)]
    pub fn is_landscape(&self) -> bool {
        matches!(self, PaperOrientation::Landscape)
    }

    /// Indicates whether this is portrait orientation.
    #[inline(always)]
    pub fn is_portrait(&self) -> bool {
        matches!(self, PaperOrientation::Portrait)
    }
}

impl TryFrom<&Cons> for Paper {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, ParseError> {
        let rest = cons.expect_cons_with_symbol_head("paper")?;
        let (paper_size_str, rest) = rest.expect_cons_with_any_str_head()?;
        if paper_size_str == "User" {
            let (height, rest) = rest.expect_cons_with_any_float_head()?;
            let (width, rest) = rest.expect_cons_with_any_float_head()?;
            rest.expect_null()?;

            let paper_size = PaperSize::User(Size::from_mm(width, height)?);

            Ok(Self {
                paper_size,
                orientation: PaperOrientation::Landscape,
            })
        } else {
            let paper_size = PaperSize::from_str(paper_size_str)?;
            let mut orientation = PaperOrientation::Landscape;

            if !rest.is_null() {
                let rest = rest.expect_cons_with_symbol_head("portrait")?;
                rest.expect_null()?;
                orientation = PaperOrientation::Portrait;
            }

            Ok(Self {
                paper_size,
                orientation,
            })
        }
    }
}

impl PaperSize {
    fn from_str(s: &str) -> Result<Self, ParseError> {
        match s {
            "A0" => Ok(Self::IsoA0),
            "A1" => Ok(Self::IsoA1),
            "A2" => Ok(Self::IsoA2),
            "A3" => Ok(Self::IsoA3),
            "A4" => Ok(Self::IsoA4),
            "A5" => Ok(Self::IsoA5),
            "A" => Ok(Self::AnsiA),
            "B" => Ok(Self::AnsiB),
            "C" => Ok(Self::AnsiC),
            "D" => Ok(Self::AnsiD),
            "E" => Ok(Self::AnsiE),
            _ => Err(ParseError::InvalidPaperSize(s.to_string())),
        }
    }
}

impl_try_from_cons_value!(Paper);
