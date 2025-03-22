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
    pub(crate) meta: Vec<Attribute>,
    pub(crate) vis: Visibility,
    pub(crate) rust_name: Ident,
    pub(crate) variants: VariantVec,
}

/// A variant within an `enum` declaration.
#[derive(Debug)]
pub(crate) struct Variant {
    pub(crate) meta: Vec<Attribute>,
    pub(crate) rust_name: Ident,
    pub(crate) sexpr_name: Ident,
}

/// A `Vec<Variant>` that can be parsed.
#[derive(Debug)]
pub(crate) struct VariantVec(Vec<Variant>);

impl EnumDecl {
    /// Generate the Rust code for the enum declaration.
    pub(crate) fn generate(&self) -> TokenStream {
        let mut result = TokenStream::new();
        result.extend(self.generate_enum_decl());
        todo!("Add impls");
    }

    /// Generate the enum decl itself.
    pub(crate) fn generate_enum_decl(&self) -> TokenStream {
        let mut result = TokenStream::new();
        let vis = &self.vis;
        let rust_name = &self.rust_name;
        let variant_decls = self.variants.generate_decls();

        for meta in &self.meta {
            result.extend(meta.to_token_stream());
        }

        result.extend(quote! {
            #vis enum #rust_name { #variant_decls }
        });

        result
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
    pub(crate) fn generate_decl(&self) -> TokenStream {
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
        let (rust_name, sexpr_name) = if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            let sexpr_name = input.parse()?;
            (name, sexpr_name)
        } else {
            (name.clone(), name)
        };

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        Ok(Self {
            meta,
            rust_name,
            sexpr_name,
        })
    }
}

impl VariantVec {
    /// Generate the variant declarations for the enum.
    pub(crate) fn generate_decls(&self) -> TokenStream {
        let mut result = TokenStream::new();
        for variant in self.iter() {
            result.extend(variant.generate_decl());
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

impl Parse for VariantVec {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let mut variants = Vec::new();

        while !input.is_empty() {
            variants.push(input.parse()?);
        }

        Ok(Self(variants))
    }
}
