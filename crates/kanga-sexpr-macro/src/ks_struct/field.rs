use {
    crate::Shape,
    proc_macro2::TokenStream,
    quote::{quote, ToTokens},
    std::{
        fmt::{Display, Formatter, Result as FmtResult},
        ops::{Deref, DerefMut},
    },
    syn::{
        braced, parenthesized,
        parse::{Parse, ParseBuffer, ParseStream, Result as ParseResult},
        Attribute, Ident, Token, Visibility,
    },
};

/// A field within a `struct` declaration.
pub(crate) struct Field {
    pub(crate) meta: Vec<Attribute>,
    pub(crate) shape: Shape,
}

/// A `Vec<[Field]>` that can be parsed.
///
/// This expects a list of fields within braces (`{}`) that denote the interior of a struct.
pub(crate) struct FieldVec(Vec<Field>);

impl Field {
    /// Generate a struct declaration for this field.
    pub(crate) fn generate_decl(&self, vis: &Visibility) -> TokenStream {
        self.shape.generate_decl(&self.meta, vis)
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        for meta in &self.meta {
            write!(f, "{} ", meta.to_token_stream())?;
        }

        write!(f, "{}", self.shape)
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let meta = input.call(Attribute::parse_outer)?;
        let shape = Shape::parse(input)?;

        Ok(Self {
            meta,
            shape,
        })
    }
}

impl Deref for FieldVec {
    type Target = [Field];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FieldVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Parse for FieldVec {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let mut fields = Vec::new();

        while !input.is_empty() {
            fields.push(input.parse()?);
        }

        Ok(Self(fields))
    }
}
