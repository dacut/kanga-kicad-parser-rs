mod field;
mod shape;

pub(crate) use self::{field::*, shape::*};

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
    pub(crate) meta: Vec<Attribute>,
    pub(crate) vis: Visibility,
    pub(crate) rust_name: Ident,
    pub(crate) sexpr_name: Ident,
    pub(crate) fields: FieldVec,
}

impl StructDecl {
    /// Generate the Rust code for the struct declaration.
    pub(crate) fn generate(&self) -> TokenStream {
        let mut result = TokenStream::new();
        result.extend(self.generate_struct_decl());
        todo!("Add impls");
    }

    /// Generate the struct decl itself.
    pub(crate) fn generate_struct_decl(&self) -> TokenStream {
        let mut result = TokenStream::new();
        let vis = &self.vis;
        let rust_name = &self.rust_name;
        let field_decls = self.generate_field_decls();

        for meta in &self.meta {
            result.extend(meta.to_token_stream());
        }

        result.extend(quote! {
            #vis struct #rust_name { #field_decls }
        });

        result
    }

    /// Generate the field declarations for the struct.
    pub(crate) fn generate_field_decls(&self) -> TokenStream {
        let mut result = TokenStream::new();

        for field in self.fields.iter() {
            result.extend(field.generate_decl(&self.vis));
        }

        result
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
        let fields: FieldVec = content.parse()?;

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
