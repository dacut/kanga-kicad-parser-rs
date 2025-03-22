use {
    crate::{impl_try_from_cons_value, LexprExt, ParseError},
    lexpr::Cons,
    serde::{Deserialize, Serialize},
    std::collections::BTreeMap,
};

/// KiCad title block.
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-intro/index.html#_title_block)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "title_block", rename_all = "snake_case")]
pub struct TitleBlock {
    /// Title
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub title: String,

    /// Date. Despite the KiCad documentation, this does not have to be a YYYY-MM-DD date (i.e.,
    /// KiCad does not validate the user input here).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub date: String,

    /// Revision
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub rev: String,

    /// Company name
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub company: String,

    /// Comments   
    #[serde(default)]
    pub comments: BTreeMap<i64, String>,
}

impl TryFrom<&Cons> for TitleBlock {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, ParseError> {
        let mut title = String::new();
        let mut date = String::new();
        let mut rev = String::new();
        let mut company = String::new();
        let mut comments = BTreeMap::new();

        let mut rest = cons.expect_cons_with_symbol_head("title_block")?;

        while !rest.is_null() {
            let cons = rest.expect_cons()?;
            let element = cons.car();
            rest = cons.cdr();
            let (key, cdr) = element.expect_cons_with_any_symbol_head()?;

            if key == "comment" {
                let (comment_id, cdr) = cdr.expect_cons_with_any_int_head()?;
                let (comment, cdr) = cdr.expect_cons_with_any_str_head()?;
                cdr.expect_null()?;

                comments.insert(comment_id, comment.to_string());
            } else {
                let (value, cdr) = cdr.expect_cons_with_any_str_head()?;
                cdr.expect_null()?;
                let value = value.to_string();

                match key {
                    "title" => title = value,
                    "date" => date = value,
                    "rev" => rev = value,
                    "company" => company = value,
                    _ => (),
                }
            }
        }

        Ok(Self {
            title,
            date,
            rev,
            company,
            comments,
        })
    }
}

impl_try_from_cons_value!(TitleBlock);
