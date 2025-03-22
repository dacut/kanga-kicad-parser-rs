use {
    proc_macro2::TokenStream,
    std::fmt::{Display, Formatter, Result as FmtResult},
    syn::{
        bracketed, parenthesized,
        parse::{Parse, ParseStream, Result as ParseResult},
        token::{Bracket, Paren},
        Attribute, Ident, Token, Visibility,
    },
};

/// The shape of an s-expression for a struct field.
#[derive(Debug)]
pub(crate) enum Shape {
    /// List of items.
    List(Vec<Shape>),

    /// Optional item.
    Option(Box<Shape>),

    /// Symbol
    Symbol(Symbol),
}

/// A symbol in an s-expression that might have a different name in Rust.
#[derive(Debug)]
pub(crate) struct Symbol {
    pub(crate) rust_name: Ident,
    pub(crate) sexpr_name: Ident,
}

impl Shape {
    /// Generate a struct field declaration for this shape.
    pub(crate) fn generate_decl(&self, meta: &[Attribute], vis: &Visibility) -> TokenStream {
        todo!("generate_decl not implemented: {meta:?} {vis:?} {self:?}");
    }

    /// If the shape is a list, return the inner items.
    #[inline(always)]
    pub(crate) fn list_inner(&self) -> Option<&[Shape]> {
        if let Shape::List(items) = self {
            Some(items)
        } else {
            None
        }
    }

    /// If the shape is an option, return the inner item.
    #[inline(always)]
    pub(crate) fn option_inner(&self) -> Option<&Shape> {
        if let Shape::Option(inner) = self {
            Some(inner)
        } else {
            None
        }
    }

    /// If the shape is a symbol, return the identifier.
    #[inline(always)]
    pub(crate) fn as_symbol(&self) -> Option<&Symbol> {
        if let Shape::Symbol(sym) = self {
            Some(sym)
        } else {
            None
        }
    }
}

impl Display for Shape {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Shape::List(items) => {
                write!(f, "(")?;
                for item in items {
                    write!(f, "{item} ")?;
                }
                write!(f, ")")
            }
            Shape::Option(inner) => {
                write!(f, "[{inner}]")
            }
            Shape::Symbol(ident) => {
                write!(f, "{ident}")
            }
        }
    }
}

impl Parse for Shape {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        if input.peek(Bracket) {
            let content;
            bracketed!(content in input);

            let inner = Self::parse(&content)?;
            Ok(Self::Option(Box::new(inner)))
        } else if input.peek(Paren) {
            let content;
            parenthesized!(content in input);

            let mut items = Vec::new();
            while !content.is_empty() {
                items.push(Self::parse(&content)?);
            }

            Ok(Self::List(items))
        } else if input.peek(Ident) {
            let sym: Symbol = input.parse()?;
            Ok(Self::Symbol(sym))
        } else {
            Err(input.error("Expected a shape (list, option, or symbol)"))
        }
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.sexpr_name)?;

        if self.rust_name != self.sexpr_name {
            write!(f, " => {}", self.rust_name)
        } else {
            Ok(())
        }
    }    
}

impl Parse for Symbol {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let sexpr_name: Ident = input.parse()?;
        let rust_name = if input.peek(Token![=>]) {
            let _: Token![=>] = input.parse()?;
            input.parse()?
        } else {
            sexpr_name.clone()
        };

        Ok(Self {
            rust_name,
            sexpr_name,
        })
    }
}

