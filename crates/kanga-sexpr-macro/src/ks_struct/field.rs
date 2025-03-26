use {
    super::{FieldMod, Shape},
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
pub(super) struct Field {
    meta: Vec<Attribute>,
    shape: Shape,
}

/// A `Vec<[Field]>` that can be parsed.
///
/// This expects a list of fields within braces (`{}`) that denote the interior of a struct.
pub(super) struct FieldVec(Vec<Field>);

impl Field {
    /// Generate a struct declaration for this field.
    pub(super) fn gen_decl(&self, vis: &Visibility) -> TokenStream {
        self.shape.gen_decl(&self.meta, vis, FieldMod::None)
    }

    /// Generate a parser for this field.
    ///
    /// The parser expects a `λv` variable, of type `lexpr::Value`, that is either a `Cons` or
    /// null. If it's a const, the `car` is the value of this field (or the next field if this
    /// field is optional and not present).
    pub(super) fn gen_parser(&self) -> TokenStream {
        self.shape.gen_parser(FieldMod::None)
    }

    /// Generate parser variable declarations for this field.
    pub(super) fn gen_parser_var_decls(&self) -> TokenStream {
        self.shape.gen_parser_var_decls(FieldMod::None)
    }
    
    /// Generate struct field setters for this field.
    pub(super) fn gen_struct_field_setters(&self) -> TokenStream {
        self.shape.gen_struct_field_setters(FieldMod::None)
    }

    /// Return the field names used for the s-expression representing this
    /// field in the struct.
    ///
    /// This depends on the shape of the field. For example, shapes like `(foo (bar:f64))` will not
    /// generate a name for `foo`, but will generate a name for `bar`.
    pub(super) fn field_names(&self) -> Vec<Ident> {
        self.shape.field_names()
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

impl<'a> IntoIterator for &'a FieldVec {
    type Item = &'a Field;
    type IntoIter = std::slice::Iter<'a, Field>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
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

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{TypeCat, TypeExt},
        pretty_assertions::assert_eq,
        quote::quote,
        syn::parse2,
    };

    #[test]
    fn test_field_basic() {
        let f: Field = parse2(quote! { [x => foo: i64] }).unwrap();
        assert!(f.meta.is_empty());
        let o = f.shape.option_inner().expect("Expected an option");
        let ts = o.as_typed_symbol().expect("Expected a typed symbol");
        assert_eq!(ts.sexpr_name, "x");
        assert_eq!(ts.rust_name, "foo");
        let Some(n) = ts.ty.as_numeric() else {
            panic!("Type is not numeric: {:?}", ts.ty);
        };
        assert_eq!(n, "i64");
    }

    #[test]
    fn test_fieldvec_basic() {
        let f: FieldVec = parse2(quote! { [x => foo: i64] y => bar: String }).unwrap();
        assert_eq!(f.len(), 2);
        let f0 = &f[0];
        assert!(f0.meta.is_empty());
        let o = f0.shape.option_inner().expect("Expected an option");
        let ts = o.as_typed_symbol().expect("Expected a typed symbol");
        assert_eq!(ts.sexpr_name, "x");
        assert_eq!(ts.rust_name, "foo");
        let Some(n) = ts.ty.as_numeric() else {
            panic!("Type is not numeric: {:?}", ts.ty);
        };
        assert_eq!(n, "i64");

        let f1 = &f[1];
        assert!(f1.meta.is_empty());
        let ts = f1.shape.as_typed_symbol().expect("Expected a typed symbol");
        assert_eq!(ts.sexpr_name, "y");
        assert_eq!(ts.rust_name, "bar");
        assert_eq!(ts.ty.category(), TypeCat::String, "Type is not string: {:?}", ts.ty);
    }
}
