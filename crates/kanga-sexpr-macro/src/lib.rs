#![allow(unused)]
mod ks_enum;
mod ks_struct;
mod type_ext;

use {
    crate::{ks_enum::*, ks_struct::*, type_ext::*},
    proc_macro::TokenStream as TokenStream1,
    proc_macro2::TokenStream,
    quote::quote,
    std::ops::Deref,
    syn::{
        parse::{Parse, ParseStream, Result as ParseResult},
        parse2, Attribute, Token,
    },
};

/// A declaration of either a struct or an enum.
pub(crate) enum Decl {
    Enum(EnumDecl),
    Struct(StructDecl),
}

/// A `Vec<[Decl]>` that can be parsed.
pub(crate) struct DeclVec(Vec<Decl>);

impl Decl {
    /// Generate the Rust code for the struct or enum.
    fn generate(&self) -> TokenStream {
        match self {
            Decl::Enum(e) => e.gen(),
            Decl::Struct(s) => s.gen(),
        }
    }
}

impl Parse for Decl {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let attr = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;

        if input.peek(Token![struct]) {
            StructDecl::parse_with_attr_vis(input, attr, vis).map(Self::Struct)
        } else if input.peek(Token![enum]) {
            EnumDecl::parse_with_attr_vis(input, attr, vis).map(Self::Enum)
        } else {
            return Err(input.error("Expected 'struct' or 'enum'"));
        }
    }
}

impl DeclVec {
    /// Generate the Rust code for the structs or enums, including the attributes.
    pub(crate) fn generate(self) -> TokenStream {
        let mut result = TokenStream::new();
        for item in self.0 {
            result.extend(item.generate());
        }

        result
    }
}

impl Deref for DeclVec {
    type Target = [Decl];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Parse for DeclVec {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let mut items = Vec::new();

        while !input.is_empty() {
            let item = Decl::parse(input)?;
            items.push(item);
        }

        Ok(Self(items))
    }
}

/// Entry point for the `#[sexpr]` attribute macro.
///
/// This just converts the `proc_macro` types into `proc_macro2` types and invokes
/// [`sexpr_impl`].
#[proc_macro]
pub fn sexpr(input: TokenStream1) -> TokenStream1 {
    sexpr_impl(input.into()).into()
}

/// The actual implementation of the `#[kicad_sexpr]` attribute macro.
///
/// This defers to the [`LexprStructEnum::parse`] method to parse the input, then generates the
/// resulting Rust code.
fn sexpr_impl(input: TokenStream) -> TokenStream {
    let decls: DeclVec = match parse2(input) {
        Ok(input) => input,
        Err(err) => return err.to_compile_error(),
    };

    decls.generate()
}

#[cfg(test)]
mod tests {
    use {super::*, pretty_assertions::assert_eq, quote::quote, syn::parse2};

    #[test]
    fn test_enum() {
        let input = quote! {
            pub enum SymbolEnum {
                symbol1
                symbol2 => Symbol2
                symbol3
            }
        };

        let decls: DeclVec = parse2(input).unwrap();
        let generated = decls.generate().to_string();
        let impl_ops = generated.find("impl").unwrap();
        let generated = generated[..impl_ops].trim();

        let expected = quote! {
            pub enum SymbolEnum {
                symbol1,
                Symbol2,
                symbol3,
            }
        };

        assert_eq!(generated.to_string(), expected.to_string());
    }

    #[test]
    fn test_struct() {
        let input = quote! {
            #[derive(Debug)]
            pub struct Color {
                (color
                    red: f64
                    green: f64
                    blue: f64
                    [alpha: f64]
                )
            }
        };

        let s: Decl = parse2(input).unwrap();
        let generated = s.generate().to_string();
        let impl_ops = generated.find("impl").unwrap();
        let generated = generated[..impl_ops].trim();

        let expected = quote! {
            #[derive(Debug)]
            pub struct Color {
                pub red: f64,
                pub green: f64,
                pub blue: f64,
                pub alpha: ::std::option::Option<f64>,
            }
        };

        assert_eq!(generated.to_string(), expected.to_string());
    }
}
