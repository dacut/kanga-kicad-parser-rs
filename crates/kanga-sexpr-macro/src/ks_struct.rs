mod field;
mod shape;

use self::{field::*, shape::*};

use {
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

/// Declaration of a `struct` as an s-expression.
pub(crate) struct StructDecl {
    meta: Vec<Attribute>,
    vis: Visibility,
    rust_name: Ident,
    sexpr_name: Ident,
    fields: FieldVec,
}

/// Types of modifiers that can be applied to a field.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum FieldMod {
    #[default]
    None,
    Optional,
    Vectored,
}

impl StructDecl {
    /// Generate the Rust code for the struct declaration.
    pub(crate) fn gen(&self) -> TokenStream {
        let mut result = TokenStream::new();
        result.extend(self.gen_struct_decl());
        result.extend(self.gen_parse_impl());
        result
    }

    /// Generate the struct decl itself.
    fn gen_struct_decl(&self) -> TokenStream {
        let mut result = TokenStream::new();
        let vis = &self.vis;
        let rust_name = &self.rust_name;
        let field_decls = self.gen_field_decls();

        for meta in &self.meta {
            result.extend(meta.to_token_stream());
        }

        result.extend(quote! {
            #vis struct #rust_name { #field_decls }
        });

        result
    }

    /// Generate the field declarations for the struct.
    fn gen_field_decls(&self) -> TokenStream {
        let mut result = TokenStream::new();

        for field in self.fields.iter() {
            result.extend(field.gen_decl(&self.vis));
        }

        result
    }

    /// Generate the parse implementation for the struct.
    fn gen_parse_impl(&self) -> TokenStream {
        let mut result = TokenStream::new();
        let rust_name = &self.rust_name;

        let mut field_parsers = TokenStream::new();
        let mut field_var_decls = TokenStream::new();
        let mut struct_field_setters = TokenStream::new();

        for field in &self.fields {
            field_var_decls.extend(field.gen_parser_var_decls());
            field_parsers.extend(field.gen_parser());
            struct_field_setters.extend(field.gen_struct_field_setters());
        }

        // We use Greek letters to avoid conflicts with field names.
        // λv = the remaining cons expression as a value
        // λ = the remaining cons expression
        // α = the car of the cons expression, our current element
        // φ = the value being extracted from the car

        quote! {
            impl ::std::convert::TryFrom<&::lexpr::Value> for #rust_name {
                type Error = ::kanga_sexpr::ParseError;

                fn try_from(mut λv: &::lexpr::Value) -> ::std::result::Result<Self, Self::Error> {
                    #field_var_decls
                    #field_parsers
                    Ok(Self { #struct_field_setters })
                }
            }
        }
    }

    /// Parse a struct declaration when the attributes and visibility have already been parsed.
    pub(crate) fn parse_with_attr_vis(input: ParseStream, meta: Vec<Attribute>, vis: Visibility) -> ParseResult<Self> {
        let _: Token![struct] = input.parse()?;
        let rust_name: Ident = input.parse()?;

        let content: ParseBuffer<'_>;
        braced!(content in input);

        let struct_outer: ParseBuffer<'_>;
        parenthesized!(struct_outer in &content);

        let sexpr_name: Ident = struct_outer.parse()?;
        let fields: FieldVec = struct_outer.parse()?;

        Ok(Self {
            meta,
            vis,
            rust_name,
            sexpr_name,
            fields,
        })
    }
}

impl Parse for StructDecl {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let meta = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        Self::parse_with_attr_vis(input, meta, vis)
    }
}

#[cfg(test)]
mod tests {
    use {super::*, crate::TypeExt, pretty_assertions::assert_eq, quote::quote, syn::parse2};

    #[test]
    fn test_basic_struct_parse() {
        let s: StructDecl = parse2(quote! { struct Foo { (foo x:i64) } }).unwrap();
    }
}
