use {
    super::FieldMod,
    crate::TypeExt,
    proc_macro2::TokenStream,
    quote::{quote, ToTokens},
    std::fmt::{Display, Formatter, Result as FmtResult},
    syn::{
        bracketed, parenthesized,
        parse::{Parse, ParseStream, Result as ParseResult, discouraged::Speculative},
        parse2,
        token::{Bracket, Paren},
        Attribute, Ident, Token, Type, Visibility,
    },
};

/// The shape of an s-expression for a struct field.
#[derive(Debug)]
pub(super) enum Shape {
    /// List of items with a symbol head.
    List(ListShape),

    /// Optional item.
    Option(Box<Shape>),

    /// Symbol without a type, used as a boolean flag.
    SymbolFlag(SymbolFlag),

    /// Symbol and type
    TypedSymbol(TypedSymbol),

    /// Vector of a shape.
    Vec(Box<Shape>),
}

/// A list shaped s-expression with a symbolic head.
#[derive(Debug)]
pub(super) struct ListShape {
    /// The symbolic head of the list.
    pub(super) sexpr_head: Ident,

    /// The Rust name to use, if renamed with `=>`.
    pub(super) rust_name: Ident,

    /// The type of the shape, if specified.
    pub(super) ty: Option<Type>,

    /// The items in the list.
    pub(super) items: Vec<Shape>,
}

/// A symbol without a type in an s-expression that might have a different name in Rust.
///
/// The s-expression syntax is `[symbol => rust_name]` or `[symbol]` if the Rust name is the same.
#[derive(Debug)]
pub(super) struct SymbolFlag {
    pub(super) rust_name: Ident,
    pub(super) sexpr_name: Ident,
}

/// A symbol and type in an s-expression that might have a different name in Rust.
#[derive(Debug)]
pub(super) struct TypedSymbol {
    pub(super) rust_name: Ident,
    pub(super) sexpr_name: Ident,
    pub(super) ty: Type,
}

impl Shape {
    /// Generate a struct field declaration for this shape.
    pub(super) fn gen_decl(&self, meta: &[Attribute], vis: &Visibility, m: FieldMod) -> TokenStream {
        match self {
            Shape::List(ls) => ls.generate_decl(meta, vis, m),
            Shape::Option(inner) => {
                assert!(m == FieldMod::None, "Cannot apply optional to field mods: {m:?}");
                inner.gen_decl(meta, vis, FieldMod::Optional)
            }
            Shape::SymbolFlag(sym) => sym.gen_decl(meta, vis, m),
            Shape::TypedSymbol(sym) => sym.gen_decl(meta, vis, m),
            Shape::Vec(inner) => {
                assert!(m == FieldMod::None, "Cannot apply vector to field mods: {m:?}");
                inner.gen_decl(meta, vis, FieldMod::Vectored)
            }
        }
    }

    /// If the shape is a list, return the inner [`ListShape`].
    #[inline(always)]
    pub(super) fn as_list_shape(&self) -> Option<&ListShape> {
        if let Shape::List(ls) = self {
            Some(ls)
        } else {
            None
        }
    }

    /// If the shape is a symbol flag, return it.
    #[inline(always)]
    pub(super) fn as_symbol_flag(&self) -> Option<&SymbolFlag> {
        if let Shape::SymbolFlag(sym) = self {
            Some(sym)
        } else {
            None
        }
    }

    /// If the shape is a typed symbol, return it.
    #[inline(always)]
    pub(super) fn as_typed_symbol(&self) -> Option<&TypedSymbol> {
        if let Shape::TypedSymbol(sym) = self {
            Some(sym)
        } else {
            None
        }
    }

    /// If the shape is optional, return the inner item.
    #[inline(always)]
    pub(super) fn option_inner(&self) -> Option<&Shape> {
        if let Shape::Option(inner) = self {
            Some(inner)
        } else {
            None
        }
    }

    /// Parse shape innards, ignoring any following '*' indicating a vectored shape.
    fn parse_non_vec(input: ParseStream) -> ParseResult<Self> {
        if input.peek(Bracket) {
            let mut content;
            bracketed!(content in input);

            // Handle the case where we have a symbol flag: `[flag]` or `[flag => rust_name]`.
            let f = content.fork();
            if let Ok(sf) = f.parse() {
                content.advance_to(&f);
                return Ok(Self::SymbolFlag(sf));
            }

            let inner = Self::parse(&content)?;
            Ok(Self::Option(Box::new(inner)))
        } else if input.peek(Paren) {
            Ok(Self::List(input.parse()?))
        } else if input.peek(Ident) {
            let sym: TypedSymbol = input.parse()?;
            Ok(Self::TypedSymbol(sym))
        } else {
            Err(input.error("Expected a shape (list, option, or symbol)"))
        }
    }
}

impl Display for Shape {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Shape::List(items) => Display::fmt(items, f),
            Shape::Option(inner) => write!(f, "[{inner}]"),
            Shape::SymbolFlag(sym) => Display::fmt(sym, f),
            Shape::TypedSymbol(ident) => Display::fmt(ident, f),
            Shape::Vec(inner) => write!(f, "{inner}*"),
        }
    }
}

impl Parse for Shape {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let inner = Self::parse_non_vec(input)?;
        if input.peek(Token![*]) {
            let _: Token![*] = input.parse()?;
            Ok(Self::Vec(Box::new(inner)))
        } else {
            Ok(inner)
        }
    }
}

impl ListShape {
    fn generate_decl(&self, meta: &[Attribute], vis: &Visibility, m: FieldMod) -> TokenStream {
        let mut result = TokenStream::new();
        let rust_name = &self.rust_name;
        let ty = &self.ty;

        if let Some(ty) = ty {
            // Type is specified; declare a struct field at this level of the list.
            assert!(self.items.is_empty(), "Type specified for list with items");

            if rust_name != "_" {
                for meta_item in meta {
                    result.extend(meta_item.to_token_stream());
                }

                result.extend(quote! { #vis #rust_name: });

                result.extend(match m {
                    FieldMod::None => quote! { #ty, },
                    FieldMod::Optional => quote! { ::std::option::Option<#ty>, },
                    FieldMod::Vectored => quote! { ::std::vec::Vec<#ty>, },
                });
            }
        } else {
            // No type specified; recurse through the list levels.
            for item in &self.items {
                result.extend(item.gen_decl(meta, vis, m));
            }
        }

        result
    }
}

impl Display for ListShape {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "({}", self.sexpr_head)?;

        if self.rust_name != self.sexpr_head {
            write!(f, " => {}", self.rust_name)?;
        }

        for item in &self.items {
            write!(f, " {}", item)?;
        }

        write!(f, ")")
    }
}

impl Parse for ListShape {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let content;
        parenthesized!(content in input);

        let sexpr_head: Ident = content.parse()?;
        let rust_name: Ident = if content.peek(Token![=>]) {
            let _: Token![=>] = content.parse()?;
            content.parse()?
        } else {
            sexpr_head.clone()
        };

        let ty;
        let mut items = Vec::new();

        if content.peek(Token![:]) {
            let _: Token![:] = content.parse()?;
            ty = Some(content.parse()?);
            if !content.is_empty() {
                return Err(content.error("Unexpected tokens after list type"));
            }
        } else {
            ty = None;
            while !content.is_empty() {
                items.push(content.parse()?);
            }
        }

        Ok(Self {
            sexpr_head,
            rust_name,
            ty,
            items,
        })
    }
}

impl SymbolFlag {
    pub(super) fn gen_decl(&self, meta: &[Attribute], vis: &Visibility, m: FieldMod) -> TokenStream {
        let rust_name = &self.rust_name;
        let mut result = TokenStream::new();

        assert!(matches!(m, FieldMod::None), "Cannot apply flag to field mods: {m:?}");

        if rust_name != "_" {
            for meta_item in meta {
                result.extend(meta_item.to_token_stream());
            }

            result.extend(quote! { #vis #rust_name: bool, });
        }

        result
    }
}

impl Display for SymbolFlag {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "[{}", self.sexpr_name)?;

        if self.rust_name != self.sexpr_name {
            write!(f, " => {}", self.rust_name)?;
        }

        write!(f, "]")
    }
}

impl Parse for SymbolFlag {
        /// Attempt to parse a `SymbolFlag` _without_ the exterior brackets.
        fn parse(input: ParseStream) -> ParseResult<Self> {
            let sexpr_name: Ident = input.parse()?;
            let rust_name: Ident = if input.peek(Token![=>]) {
                let _: Token![=>] = input.parse()?;
                input.parse()?
            } else {
                sexpr_name.clone()
            };
    
            if !input.is_empty() {
                return Err(input.error("Unexpected tokens after symbol flag"));
            }
    
            Ok(Self {
                rust_name,
                sexpr_name,
            })
        }
        
}
impl TypedSymbol {
    pub(super) fn gen_decl(&self, meta: &[Attribute], vis: &Visibility, m: FieldMod) -> TokenStream {
        let ty = &self.ty;
        let rust_name = &self.rust_name;
        let mut result = TokenStream::new();

        if rust_name != "_" {
            for meta_item in meta {
                result.extend(meta_item.to_token_stream());
            }

            result.extend(quote! {#vis #rust_name: });
            result.extend(match m {
                FieldMod::None => quote! { #ty, },
                FieldMod::Optional => quote! { ::std::option::Option<#ty>, },
                FieldMod::Vectored => quote! { ::std::vec::Vec<#ty>, },
            })
        }

        result
    }
}

impl Display for TypedSymbol {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.sexpr_name)?;

        if self.rust_name != self.sexpr_name {
            write!(f, " => {}", self.rust_name)?;
        }

        write!(f, ": {}", self.ty.to_string())
    }
}

impl Parse for TypedSymbol {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let sexpr_name: Ident = input.parse()?;
        let rust_name: Ident = if input.peek(Token![=>]) {
            let _: Token![=>] = input.parse()?;
            input.parse()?
        } else {
            sexpr_name.clone()
        };

        let _: Token![:] = input.parse()?;
        let ty: Type = input.parse()?;

        Ok(Self {
            rust_name,
            sexpr_name,
            ty,
        })
    }
}

#[cfg(test)]
mod tests {
    use {super::*, crate::TypeExt, pretty_assertions::assert_eq, quote::quote, syn::parse2};

    /// Basic typed symbol parsing.
    #[test]
    fn typed_symbol_good() {
        let s: Shape = parse2(quote! { sexpr => rust: f64 }).unwrap();
        let ts = s.as_typed_symbol().expect("Expected a typed symbol");
        assert_eq!(ts.sexpr_name, "sexpr");
        assert_eq!(ts.rust_name, "rust");
        let Some(n) = ts.ty.as_numeric() else {
            panic!("Type is not numeric: {:?}", ts.ty);
        };
        assert_eq!(n, "f64");
    }

    /// List shape parsing
    #[test]
    fn list_shape_good() {
        let s: Shape = parse2(quote! { (hello world:f64) }).unwrap();
        let ls = s.as_list_shape().expect("Expected a list shape");
        assert_eq!(ls.sexpr_head, "hello");
        assert_eq!(ls.rust_name, "hello");
        assert_eq!(ls.items.len(), 1);
        let i0 = &ls.items[0];
        let i0 = i0.as_typed_symbol().expect("Expected a typed symbol");
        assert_eq!(i0.sexpr_name, "world");
        assert_eq!(i0.rust_name, "world");
        let Some(n) = i0.ty.as_numeric() else {
            panic!("Type is not numeric: {:?}", i0.ty);
        };
        assert_eq!(n, "f64");
    }

    /// Option parsing
    #[test]
    fn option_typed_shape_good() {
        let s: Shape = parse2(quote! { [hello => world: f64] }).unwrap();
        let o = s.option_inner().expect("Expected an option");
        let ts = o.as_typed_symbol().expect("Expected a typed symbol");
        assert_eq!(ts.sexpr_name, "hello");
        assert_eq!(ts.rust_name, "world");
        let Some(n) = ts.ty.as_numeric() else {
            panic!("Type is not numeric: {:?}", ts.ty);
        };
        assert_eq!(n, "f64");
    }

    #[test]
    fn option_list_shape_good() {
        let s: Shape = parse2(quote! { [(hello => world x => foo: i64 [y => bar: String])]}).unwrap();
        let o = s.option_inner().expect("Expected an option");
        let Shape::List(ls) = &o else {
            panic!("Not a list shape: {:?}", o);
        };
        assert_eq!(ls.sexpr_head, "hello");
        assert_eq!(ls.rust_name, "world");
        assert_eq!(ls.items.len(), 2);
        let i0 = &ls.items[0];
        let i0 = i0.as_typed_symbol().expect("Expected a typed symbol");
        let i1 = &ls.items[1];
        let i1 = i1.option_inner().expect("Expected an option");
        let i1 = i1.as_typed_symbol().expect("Expected a typed symbol");
        assert_eq!(i0.sexpr_name, "x");
        assert_eq!(i0.rust_name, "foo");
        let Some(n) = i0.ty.as_numeric() else {
            panic!("Type is not numeric: {:?}", i0.ty);
        };
        assert_eq!(n, "i64");
        assert_eq!(i1.sexpr_name, "y");
        assert_eq!(i1.rust_name, "bar");
        assert!(i1.ty.is_string(), "Not a string: {:?}", i1.ty);
    }

    #[test]
    fn symbol_flag_good() {
        let s: Shape = parse2(quote! { [hello] }).unwrap();
        let sf = s.as_symbol_flag().expect("Expected a symbol flag");
        assert_eq!(sf.sexpr_name, "hello");
        assert_eq!(sf.rust_name, "hello");
    }
}
