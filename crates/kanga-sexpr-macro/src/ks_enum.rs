use {
    proc_macro2::TokenStream,
    quote::{quote, ToTokens},
    std::{
        fmt::{Display, Formatter, Result as FmtResult},
        iter::{IntoIterator, Iterator},
        ops::{Deref, DerefMut},
    },
    syn::{
        braced,
        parse::{Parse, ParseStream, Result as ParseResult},
        Attribute, Ident, Token, Visibility,
    },
};

/// Declaration of an `enum` as an s-expression.
#[derive(Debug)]
pub(crate) struct EnumDecl {
    meta: Vec<Attribute>,
    vis: Visibility,
    rust_name: Ident,
    variants: VariantVec,
}

/// A variant within an `enum` declaration.
#[derive(Debug)]
struct Variant {
    meta: Vec<Attribute>,
    sexpr_name: Ident,
    rust_name: Ident,
}

/// A `Vec<Variant>` that can be parsed.
#[derive(Debug)]
struct VariantVec(Vec<Variant>);

impl EnumDecl {
    /// Generate the Rust code for the enum declaration.
    pub(crate) fn gen(&self) -> TokenStream {
        let mut result = TokenStream::new();
        result.extend(self.gen_enum_decl());
        result.extend(self.gen_parse_impl());
        result
    }

    /// Generate the enum decl itself.
    fn gen_enum_decl(&self) -> TokenStream {
        let mut result = TokenStream::new();
        let vis = &self.vis;
        let rust_name = &self.rust_name;
        let variant_decls = self.variants.gen_decls();

        for meta in &self.meta {
            result.extend(meta.to_token_stream());
        }

        result.extend(quote! {
            #vis enum #rust_name { #variant_decls }
        });

        result
    }

    /// Generate the parse implementation for the enum.
    fn gen_parse_impl(&self) -> TokenStream {
        let mut result = TokenStream::new();
        let rust_name = &self.rust_name;
        let mut enum_expected = TokenStream::new(); // The expected symbols for the enum.
        let mut match_arms = TokenStream::new();    // Handlers for the `match sym` statement.

        for variant in &self.variants {
            // Add this variant's sexpr name to the array of expected symbols for the enum.
            let sexpr_name = variant.sexpr_name.to_string();
            let rust_name = &variant.rust_name;
            enum_expected.extend(quote! { #sexpr_name, });

            // Add a match arm for this variant.
            match_arms.extend(quote! {
                #sexpr_name => Ok(Self::#rust_name),
            })
        }

        quote! {
            impl ::std::convert::TryFrom<&::lexpr::Value> for #rust_name {
                type Error = ::kanga_sexpr::ParseError;

                fn try_from(value: &::lexpr::Value) -> ::std::result::Result<Self, Self::Error> {
                    const EXPECTED: &'static [&'static str] = &[#enum_expected];

                    let Some(sym) = value.as_symbol() else {
                        return Err(::kanga_sexpr::ParseError::ExpectedEnumSymbol(value.clone(), EXPECTED));
                    };

                    match sym {
                        #match_arms
                        _ => Err(::kanga_sexpr::ParseError::ExpectedEnumSymbol(value.clone(), EXPECTED)),
                    }
                }
            }
        }
        
    }

    /// Parse a struct declaration when the attributes and visibility have already been parsed.
    pub(crate) fn parse_with_attr_vis(input: ParseStream, meta: Vec<Attribute>, vis: Visibility) -> ParseResult<Self> {
        let _: Token![enum] = input.parse()?;
        let rust_name: Ident = input.parse()?;

        let content;
        braced!(content in input);
        let variants = content.parse()?;

        Ok(Self {
            meta,
            vis,
            rust_name,
            variants,
        })
    }
}

impl Parse for EnumDecl {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let meta = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        Self::parse_with_attr_vis(input, meta, vis)
    }
}

impl Variant {
    fn gen_decl(&self) -> TokenStream {
        let mut result = TokenStream::new();
        for meta in &self.meta {
            result.extend(meta.to_token_stream());
        }

        let rust_name = &self.rust_name;
        result.extend(quote! {
            #rust_name,
        });

        result
    }
}

impl Display for Variant {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if self.rust_name == self.sexpr_name {
            write!(f, "{}", self.rust_name)
        } else {
            write!(f, "{} => {}", self.sexpr_name, self.rust_name)
        }
    }
}

impl Parse for Variant {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let meta = input.call(Attribute::parse_outer)?;
        let name: Ident = input.parse()?;
        let (sexpr_name, rust_name) = if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            let rust_name = input.parse()?;
            (name, rust_name)
        } else {
            (name.clone(), name)
        };

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        Ok(Self {
            meta,
            sexpr_name,
            rust_name,
        })
    }
}

impl VariantVec {
    /// Generate the variant declarations for the enum.
    fn gen_decls(&self) -> TokenStream {
        let mut result = TokenStream::new();
        for variant in self.iter() {
            result.extend(variant.gen_decl());
        }
        result
    }
}

impl Deref for VariantVec {
    type Target = Vec<Variant>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VariantVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for VariantVec {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "(")?;
        for (i, variant) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{variant}")?;
        }
        write!(f, ")")
    }
}

impl IntoIterator for VariantVec {
    type Item = Variant;
    type IntoIter = std::vec::IntoIter<Variant>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a VariantVec {
    type Item = &'a Variant;
    type IntoIter = std::slice::Iter<'a, Variant>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Parse for VariantVec {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let mut variants = Vec::new();

        while !input.is_empty() {
            variants.push(input.parse()?);
        }

        Ok(Self(variants))
    }
}
