use {
    super::{Font, TextJustify},
    crate::{impl_try_from_cons_value, LexprExt, ParseError},
    lexpr::Cons,
    serde::{Deserialize, Serialize},
};

/// KiCad text effects definition.
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-intro/index.html#_text_effects)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "effects")]
pub struct TextEffects {
    /// Text font.
    pub font: Option<Font>,

    /// Text justification.
    pub justify: Option<TextJustify>,

    /// Whether the text is hidden.
    #[serde(default)]
    pub hide: bool,
}

impl TryFrom<&Cons> for TextEffects {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, Self::Error> {
        let mut rest = cons.expect_cons_with_symbol_head("effects")?;
        let mut font = None;
        let mut justify = None;
        let mut hide = false;

        while !rest.is_null() {
            let cons = rest.expect_cons()?;
            let element = cons.car();
            rest = cons.cdr();

            if let Some(e_cons) = element.as_cons() {
                let (key, _) = e_cons.expect_cons_with_any_symbol_head()?;

                match key {
                    "font" => font = Some(Font::try_from(e_cons)?),
                    "justify" => justify = Some(TextJustify::try_from(e_cons)?),
                    _ => return Err(ParseError::Unexpected(element.clone())),
                }
            } else if let Some(sym) = element.as_symbol() {
                if sym == "hide" {
                    hide = true;
                } else {
                    return Err(ParseError::Unexpected(element.clone()));
                }
            }
        }

        Ok(Self {
            font,
            justify,
            hide,
        })
    }
}

impl_try_from_cons_value!(TextEffects);
