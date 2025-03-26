use {
    super::FieldMod,
    crate::{TypeCat, TypeExt},
    proc_macro2::TokenStream,
    quote::{quote, ToTokens},
    std::fmt::{Display, Formatter, Result as FmtResult},
    syn::{
        bracketed, parenthesized,
        parse::{discouraged::Speculative, Parse, ParseStream, Result as ParseResult},
        parse2,
        token::{Bracket, Paren},
        Attribute, Ident, Token, Type, Visibility,
    },
};

/// The shape of an s-expression for a struct field.
#[derive(Debug)]
pub(super) enum Shape {
    /// List of items with a symbol head whose contents are destructured into struct fields.
    DesList(DesList),

    /// List with a symbol head represented by a type.
    TypedList(TypedList),

    /// Optional item.
    Option(Box<Shape>),

    /// Symbol without a type, used as a boolean flag.
    SymbolFlag(SymbolFlag),

    /// Symbol and type
    TypedSymbol(TypedSymbol),

    /// Vector of a shape.
    Vec(Box<Shape>),
}

/// List of items with a symbol head whose contents are destructured into struct fields.
#[derive(Debug)]
pub(super) struct DesList {
    /// The symbolic head of the list.
    pub(super) sexpr_head: Ident,

    /// The items in the list.
    pub(super) items: Vec<Shape>,
}

/// List of items with a symbol head represented by a type.
#[derive(Debug)]
pub(super) struct TypedList {
    /// The symbolic head of the list
    pub(super) sexpr_head: Ident,

    /// The Rust name to use, if renamed with `=>`.
    pub(super) rust_name: Ident,

    /// The type of the list.
    pub(super) ty: Type,
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
            Shape::DesList(ls) => ls.gen_decl(meta, vis, m),
            Shape::TypedList(ls) => ls.gen_decl(meta, vis, m),
            Shape::Option(inner) => {
                assert!(m == FieldMod::None, "Cannot apply field mod {m:?} to optional shape");
                inner.gen_decl(meta, vis, FieldMod::Optional)
            }
            Shape::SymbolFlag(sym) => sym.gen_decl(meta, vis, m),
            Shape::TypedSymbol(sym) => {
                assert!(m != FieldMod::Vectored, "Cannot apply field mod {m:?} to typed symbol");
                sym.gen_decl(meta, vis, m)
            }
            Shape::Vec(inner) => {
                assert!(m == FieldMod::None, "Cannot apply field mod {m:?} to vectored shape");
                inner.gen_decl(meta, vis, FieldMod::Vectored)
            }
        }
    }

    /// Generate a parser for this shape.
    ///
    /// The parser expects a `cons` variable, of type `lexpr::Cons`, whose `car` is the value of
    /// this field (or the next field if this field is optional and not present).
    pub(super) fn gen_parser(&self, m: FieldMod) -> TokenStream {
        match self {
            Self::DesList(dl) => dl.gen_parser(m),
            Self::TypedList(tl) => tl.gen_parser(m),
            Self::Option(inner) => {
                assert!(m == FieldMod::None, "Cannot apply field mod {m:?} to optional shape");
                inner.gen_parser(FieldMod::Optional)
            }
            Self::SymbolFlag(sym) => sym.gen_parser(m),
            Self::TypedSymbol(sym) => sym.gen_parser(m),
            Self::Vec(inner) => {
                assert!(m == FieldMod::None, "Cannot apply field mod {:?} to vectored shape", m);
                inner.gen_parser(FieldMod::Vectored)
            }
        }
    }

    /// Generate variable declarations for this field.
    pub(super) fn gen_parser_var_decls(&self, m: FieldMod) -> TokenStream {
        match self {
            Self::DesList(dl) => dl.gen_parser_var_decls(m),
            Self::TypedList(tl) => tl.gen_parser_var_decls(m),
            Self::Option(inner) => {
                assert!(m == FieldMod::None, "Cannot apply field mod {m:?} to optional shape");
                inner.gen_parser_var_decls(FieldMod::Optional)
            }
            Self::SymbolFlag(sym) => sym.gen_parser_var_decls(m),
            Self::TypedSymbol(sym) => sym.gen_parser_var_decls(m),
            Self::Vec(inner) => {
                assert!(m == FieldMod::None, "Cannot apply field mod {m:?} to vectored shape");
                inner.gen_parser_var_decls(FieldMod::Vectored)
            }
        }
    }

    /// Generate struct field setters for this shape.
    pub(super) fn gen_struct_field_setters(&self, m: FieldMod) -> TokenStream {
        match self {
            Self::DesList(dl) => dl.gen_struct_field_setters(m),
            Self::TypedList(tl) => tl.gen_struct_field_setters(m),
            Self::Option(inner) => {
                assert!(m == FieldMod::None, "Cannot apply field mod {m:?} to optional shape");
                inner.gen_struct_field_setters(FieldMod::Optional)
            }
            Self::SymbolFlag(sym) => sym.gen_struct_field_setters(m),
            Self::TypedSymbol(sym) => sym.gen_struct_field_setters(m),
            Self::Vec(inner) => {
                assert!(m == FieldMod::None, "Cannot apply field mod {m:?} to vectored shape");
                inner.gen_struct_field_setters(FieldMod::Vectored)
            }
        }
    }

    /// Return the field names used for the s-expression representing this shape.
    pub(super) fn field_names(&self) -> Vec<Ident> {
        match self {
            Shape::DesList(ls) => ls.field_names(),
            Shape::TypedList(ls) => ls.field_names(),
            Shape::Option(inner) => inner.field_names(),
            Shape::SymbolFlag(sym) => sym.field_names(),
            Shape::TypedSymbol(sym) => sym.field_names(),
            Shape::Vec(inner) => inner.field_names(),
        }
    }

    /// If the shape is a list, return the inner [`ListShape`].
    pub(super) fn as_list_shape(&self) -> Option<&DesList> {
        if let Shape::DesList(ls) = self {
            Some(ls)
        } else {
            None
        }
    }

    /// If the shape is a symbol flag, return it.
    pub(super) fn as_symbol_flag(&self) -> Option<&SymbolFlag> {
        if let Shape::SymbolFlag(sym) = self {
            Some(sym)
        } else {
            None
        }
    }

    /// If the shape is a typed symbol, return it.
    pub(super) fn as_typed_symbol(&self) -> Option<&TypedSymbol> {
        if let Shape::TypedSymbol(sym) = self {
            Some(sym)
        } else {
            None
        }
    }

    /// If the shape is optional, return the inner item.
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
            if matches!(inner, Shape::SymbolFlag(_)) {
                Ok(inner)
            } else {
                Ok(Self::Option(Box::new(inner)))
            }
        } else if input.peek(Paren) {
            let content;
            parenthesized!(content in input);

            // Determine whether we have a typed list (`(head => rust_name: type)`) or a
            // destructured list (`(head => rust_name item1 item2)`)
            let f = content.fork();
            let _sexpr_name: Ident = f.parse()?;
            if f.peek(Token![=>]) {
                let _: Token![=>] = f.parse()?;
                let _rust_name: Ident = f.parse()?;
            }

            if f.peek(Token![:]) {
                // This is a typed list
                Ok(Self::TypedList(content.parse()?))
            } else {
                // This is a destructured list
                Ok(Self::DesList(content.parse()?))
            }
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
            Shape::DesList(items) => Display::fmt(items, f),
            Shape::TypedList(items) => Display::fmt(items, f),
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

impl DesList {
    fn gen_decl(&self, meta: &[Attribute], vis: &Visibility, m: FieldMod) -> TokenStream {
        let mut result = TokenStream::new();
        for item in &self.items {
            result.extend(item.gen_decl(meta, vis, m));
        }

        result
    }

    /// Generate a parser for this destructured list.
    fn gen_parser(&self, m: FieldMod) -> TokenStream {
        let sexpr_head = &self.sexpr_head;
        let mut item_parsers = TokenStream::new();
        for item in &self.items {
            item_parsers.extend(item.gen_parser(m));
        }

        let not_list_else = match m {
            FieldMod::None => quote! {
                else { return Err(::kanga_sexpr::ParseError::ExpectedList(λv.clone())); }
            },
            _ => quote! {},
        };

        let not_sym_else = match m {
            FieldMod::None => quote! {
                else { return Err(::kanga_sexpr::ParseError::ExpectedSym(α.clone())); }
            },
            _ => quote! {},
        };

        quote! {
            if let Some(λ) = λv.as_cons() {
                let α = λ.car();
                if α.as_symbol() == Some(stringify!(#sexpr_head)) {
                    {
                        let mut λv = λ.cdr();
                        #item_parsers
                    }
                    λv = λ.cdr();
                }
                #not_sym_else
            }
            #not_list_else
        }
    }

    /// Generate variable declarations for this destructured list.
    fn gen_parser_var_decls(&self, m: FieldMod) -> TokenStream {
        let mut result = TokenStream::new();
        for item in &self.items {
            result.extend(item.gen_parser_var_decls(m));
        }
        result
    }

    /// Generate struct field setters for this destructured list.
    fn gen_struct_field_setters(&self, m: FieldMod) -> TokenStream {
        let mut result = TokenStream::new();
        for item in &self.items {
            result.extend(item.gen_struct_field_setters(m));
        }
        result
    }

    /// Return the field names used for the s-expression representing this list shape.
    fn field_names(&self) -> Vec<Ident> {
        let mut result = Vec::new();
        for item in &self.items {
            result.extend(item.field_names());
        }
        result
    }
}

impl Display for DesList {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "({}", self.sexpr_head)?;

        for item in &self.items {
            write!(f, " {}", item)?;
        }

        write!(f, ")")
    }
}

impl Parse for DesList {
    /// Parse a `DesList` shape from the input. This assumes the outer parentheses have already been consumed.
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let sexpr_head: Ident = input.parse()?;
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
        }

        Ok(Self {
            sexpr_head,
            items,
        })
    }
}

impl TypedList {
    fn gen_decl(&self, meta: &[Attribute], vis: &Visibility, m: FieldMod) -> TokenStream {
        let ty = &self.ty;
        let rust_name = &self.rust_name;
        let mut result = TokenStream::new();

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

        result
    }

    /// Generate a parser for this typed list.
    fn gen_parser(&self, m: FieldMod) -> TokenStream {
        match m {
            FieldMod::None => self.gen_std_parser(),
            FieldMod::Optional => self.gen_optional_parser(),
            FieldMod::Vectored => self.gen_vectored_parser(),
        }
    }

    /// Generate parser variable declarations for this typed list.
    fn gen_parser_var_decls(&self, m: FieldMod) -> TokenStream {
        let rust_name = &self.rust_name;
        if rust_name != "_" {
            match m {
                FieldMod::Vectored => quote! { let mut #rust_name = Vec::new(); },
                _ => quote! { let #rust_name; },
            }
        } else {
            quote! {}
        }
    }

    /// Generate struct field setters for this typed list.
    fn gen_struct_field_setters(&self, m: FieldMod) -> TokenStream {
        let rust_name = &self.rust_name;
        if rust_name != "_" {
            quote! { #rust_name, }
        } else {
            quote! {}
        }
    }

    /// Generate a standard parser for this typed list.
    fn gen_std_parser(&self) -> TokenStream {
        let sexpr_name = &self.sexpr_head;
        let rust_name = &self.rust_name;
        let ty = &self.ty;

        let field_parser = match ty.category() {
            TypeCat::Float => quote! {
                α.as_f64().ok_or(::kanga_sexpr::ParseError::ExpectedFloat(α.clone()))?
            },
            TypeCat::Int => quote! {
                α.as_i64().ok_or(::kanga_sexpr::ParseError::ExpectedInt(α.clone()))?
            },
            TypeCat::String => quote! {
                α.as_str().ok_or(::kanga_sexpr::ParseError::ExpectedStr(α.clone()))?.to_string()
            },
            TypeCat::Uuid => quote! {
                Uuid::from_str(
                    α.as_str().ok_or(::kanga_sexpr::ParseError::ExpectedUuid(α.clone()))?)
                    .map_err(|_| ::kanga_sexpr::ParseError::ExpectedUuid(α.clone()))?
            },
            TypeCat::General => quote! {
                #ty::try_from(α)?
            },
            TypeCat::Unsupported => panic!("Unsupported type category for typed list: {:?}", ty),
        };

        quote! {
            let Some(λ) = λv.as_cons() else {
                return Err(::kanga_sexpr::ParseError::ExpectedList(λv.clone()));
            };

            let α = λ.car();
            if α.as_symbol() == Some(stringify!(#sexpr_name)) {
                #rust_name = #field_parser;
                λv = λ.cdr();
            } else {
                return Err(::kanga_sexpr::ParseError::ExpectedSym(α.clone()));
            }
            drop(α);
            drop(λ);
        }
    }

    fn gen_optional_parser(&self) -> TokenStream {
        let sexpr_name = &self.sexpr_head;
        let rust_name = &self.rust_name;
        let ty = &self.ty;

        let field_parser = match ty.category() {
            TypeCat::Float => quote! {
                α.as_f64().ok_or(::kanga_sexpr::ParseError::ExpectedFloat(α.clone()))?
            },
            TypeCat::Int => quote! {
                α.as_i64().ok_or(::kanga_sexpr::ParseError::ExpectedInt(α.clone()))?
            },
            TypeCat::String => quote! {
                α.as_str().ok_or(::kanga_sexpr::ParseError::ExpectedStr(α.clone()))?.to_string()
            },
            TypeCat::Uuid => quote! {
                Uuid::from_str(
                    α.as_str().ok_or(::kanga_sexpr::ParseError::ExpectedUuid(α.clone()))?)
                    .map_err(|_| ::kanga_sexpr::ParseError::ExpectedUuid(α.clone()))?
            },
            TypeCat::General => quote! {
                #ty::try_from(α)?
            },
            TypeCat::Unsupported => panic!("Unsupported type category for typed list: {:?}", ty),
        };

        quote! {
            if let Some(λ) = λv.as_cons() {
                let α = λ.car();

                if α.as_symbol() == Some(stringify!(#sexpr_name)) {
                    #rust_name = Some(#field_parser);
                    λv = λ.cdr();
                } else {
                    #rust_name = None;
                }
            } else {
                #rust_name = None;
            }
        }
    }

    fn gen_vectored_parser(&self) -> TokenStream {
        let sexpr_name = &self.sexpr_head;
        let rust_name = &self.rust_name;
        let ty = &self.ty;

        let field_parser = match ty.category() {
            TypeCat::Float => quote! {
                α.as_f64().ok_or(::kanga_sexpr::ParseError::ExpectedFloat(α.clone()))?
            },
            TypeCat::Int => quote! {
                α.as_i64().ok_or(::kanga_sexpr::ParseError::ExpectedInt(α.clone()))?
            },
            TypeCat::String => quote! {
                α.as_str().ok_or(::kanga_sexpr::ParseError::ExpectedStr(α.clone()))?.to_string()
            },
            TypeCat::Uuid => quote! {
                Uuid::from_str(
                    α.as_str().ok_or(::kanga_sexpr::ParseError::ExpectedUuid(α.clone()))?)
                    .map_err(|_| ::kanga_sexpr::ParseError::ExpectedUuid(α.clone()))?
            },
            TypeCat::General => quote! {
                #ty::try_from(α)?
            },
            TypeCat::Unsupported => panic!("Unsupported type category for typed list: {:?}", ty),
        };

        quote! {
            // TypedList::gen_vectored_parser
            if let Some(λ) = λv.as_cons() {
                let α = λ.car();

                if α.as_symbol() == Some(stringify!(#sexpr_name)) {
                    #rust_name.push(#field_parser);
                    λv = λ.cdr();
                }
            }
        }
    }

    /// Return the field names used for the s-expression representing this list shape.
    fn field_names(&self) -> Vec<Ident> {
        if self.rust_name == "_" {
            vec![]
        } else {
            vec![self.sexpr_head.clone()]
        }
    }
}

impl Display for TypedList {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "({}", self.sexpr_head)?;

        if self.rust_name != self.sexpr_head {
            write!(f, " => {}", self.rust_name)?;
        }

        write!(f, ": {}", self.ty.to_token_stream())?;

        write!(f, ")")
    }
}

impl Parse for TypedList {
    /// Parse a `DesList` shape from the input. This assumes the outer parentheses have already been consumed.
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let sexpr_head: Ident = input.parse()?;
        let rust_name: Ident = if input.peek(Token![=>]) {
            let _: Token![=>] = input.parse()?;
            input.parse()?
        } else {
            sexpr_head.clone()
        };

        let _: Token![:] = input.parse()?;
        let ty: Type = input.parse()?;

        if !input.is_empty() {
            return Err(input.error("Unexpected tokens after typed list"));
        }

        Ok(Self {
            sexpr_head,
            rust_name,
            ty,
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

    /// Generate a parser for this symbol flag.
    fn gen_parser(&self, m: FieldMod) -> TokenStream {
        assert_eq!(m, FieldMod::None, "Cannot apply field mod {m:?} to symbol flag");
        let sexpr_name = &self.sexpr_name;
        let rust_name = &self.rust_name;

        match m {
            FieldMod::None => quote! {
                let Some(λ) = λv.as_cons() else {
                    return Err(::kanga_sexpr::ParseError::ExpectedList(λv.clone()));
                };
                let α = λ.car();
                if α.as_symbol() == Some(stringify!(#sexpr_name)) {
                    #rust_name = true;
                    λv = λ.cdr();
                } else {
                    #rust_name = false;
                }
    
                drop(α);
                drop(λ);
            },
            FieldMod::Optional => quote! {
                if let Some(λ) = λv.as_cons() {
                    let α = λ.car();
                    if α.as_symbol() == Some(stringify!(#sexpr_name)) {
                        #rust_name = true;
                        λv = λ.cdr();
                    } else {
                        #rust_name = false;
                    }
                } else {
                    #rust_name = false;
                }
            },
            _ => panic!("Cannot apply field mod {m:?} to symbol flag"),
        }
    }

    /// Generate parser variable declarations for this symbol flag.
    fn gen_parser_var_decls(&self, m: FieldMod) -> TokenStream {
        assert_eq!(m, FieldMod::None, "Cannot apply field mod {m:?} to symbol flag");
        let rust_name = &self.rust_name;
        if rust_name != "_" {
            quote! { let #rust_name; }
        } else {
            quote! {}
        }
    }

    /// Generate struct field setters for this symbol flag.
    fn gen_struct_field_setters(&self, m: FieldMod) -> TokenStream {
        assert_eq!(m, FieldMod::None, "Cannot apply field mod {m:?} to symbol flag");
        let rust_name = &self.rust_name;
        if rust_name != "_" {
            quote! { #rust_name, }
        } else {
            quote! {}
        }
    }

    /// Return the field names used for the s-expression representing this list shape.
    fn field_names(&self) -> Vec<Ident> {
        if self.rust_name == "_" {
            vec![]
        } else {
            vec![self.sexpr_name.clone()]
        }
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

    /// Generate a parser for this typed symbol.
    fn gen_parser(&self, m: FieldMod) -> TokenStream {
        match m {
            FieldMod::None => self.gen_std_parser(),
            FieldMod::Optional => self.gen_optional_parser(),
            FieldMod::Vectored => self.gen_vectored_parser(),
        }
    }

    fn gen_std_parser(&self) -> TokenStream {
        let sexpr_name = &self.sexpr_name;
        let rust_name = &self.rust_name;
        let ty = &self.ty;

        let ty_parser = match ty.category() {
            TypeCat::Float => quote! {
                if let Some(φ) = α.as_f64() {
                    #rust_name = φ;
                    λv = λ.cdr();
                } else {
                    return Err(::kanga_sexpr::ParseError::ExpectedFloat(α.clone()));
                }
            },
            TypeCat::Int => quote! {
                if let Some(φ) = α.as_i64() {
                    #rust_name = φ;
                    λv = λ.cdr();
                } else {
                    return Err(::kanga_sexpr::ParseError::ExpectedInt(α.clone()));
                }
            },
            TypeCat::String => quote! {
                if let Some(φ) = α.as_str() {
                    #rust_name = φ.to_string();
                    λv = λ.cdr();
                } else {
                    return Err(::kanga_sexpr::ParseError::ExpectedStr(α.clone()));
                }
            },
            TypeCat::Uuid => quote! {
                if let Some(φ) = α.as_str() {
                    match ::uuid::Uuid::parse_str(φ) {
                        Ok(φ) => {
                            #rust_name = φ;
                            λv = λ.cdr();
                        },
                        Err(_) => return Err(::kanga_sexpr::ParseError::ExpectedUuid(α.clone())),
                    }
                } else {
                    return Err(::kanga_sexpr::ParseError::ExpectedUuid(α.clone()));
                }
            },
            TypeCat::General => quote! {
                #rust_name = #ty :: try_from(α)?;
                λv = λ.cdr();
            },
            TypeCat::Unsupported => panic!("Unsupported type category for typed symbol: {:?}", ty),
        };
        quote! {
            let Some(λ) = λv.as_cons() else {
                return Err(::kanga_sexpr::ParseError::ExpectedList(λv.clone()));
            };
            let α = λ.car();
            #ty_parser
            drop(α);
            drop(λ);
        }
    }

    fn gen_optional_parser(&self) -> TokenStream {
        let sexpr_name = &self.sexpr_name;
        let rust_name = &self.rust_name;
        let ty = &self.ty;

        let ty_parser = match ty.category() {
            TypeCat::Float => quote! {
                if let Some(φ) = α.as_f64() {
                    #rust_name = Some(φ);
                    λv = λ.cdr();
                } else {
                    #rust_name = None;
                }
            },
            TypeCat::Int => quote! {
                if let Some(φ) = α.as_i64() {
                    #rust_name = Some(φ);
                    λv = λ.cdr();
                } else {
                    #rust_name = None;
                }
            },
            TypeCat::String => quote! {
                if let Some(φ) = α.as_str() {
                    #rust_name = Some(φ.to_string());
                    λv = λ.cdr();
                } else {
                    #rust_name = None;
                }
            },
            TypeCat::Uuid => quote! {
                if let Some(φ) = α.as_str() {
                    match ::uuid::Uuid::parse_str(φ) {
                        Ok(φ) => {
                            #rust_name = Some(φ);
                            λv = λ.cdr();
                        }
                        Err(_) => {
                            #rust_name = None;
                        }
                    }
                } else {
                    #rust_name = None;
                }
            },
            TypeCat::General => quote! {
                if let Ok(φ) = #ty::try_from(α) {
                    #rust_name = Some(φ);
                    λv = λ.cdr();
                } else {
                    #rust_name = None;
                }
            },
            TypeCat::Unsupported => panic!("Unsupported type category for typed symbol: {:?}", ty),
        };

        quote! {
            if let Some(λ) = λv.as_cons() {
                let α = λ.car();
                #ty_parser
                drop(α);
                drop(λ);
            } else {
                #rust_name = None;
            }
        }
    }

    fn gen_vectored_parser(&self) -> TokenStream {
        let sexpr_name = &self.sexpr_name;
        let rust_name = &self.rust_name;
        let ty = &self.ty;

        let ty_parser = match ty.category() {
            TypeCat::Float => quote! {
                if let Some(φ) = α.as_f64() {
                    #rust_name.push(φ);
                    λv = λ.cdr();
                }
            },
            TypeCat::Int => quote! {
                if let Some(φ) = α.as_i64() {
                    #rust_name.push(φ);
                    λv = λ.cdr();
                }
            },
            TypeCat::String => quote! {
                if let Some(φ) = α.as_str() {
                    #rust_name.push(φ.to_string());
                    λv = λ.cdr();
                }
            },
            TypeCat::Uuid => quote! {
                if let Some(φ) = α.as_str() {
                    if let Ok(φ) = ::uuid::Uuid::parse_str(φ) {
                        #rust_name.push(φ);
                        λv = λ.cdr();
                    }
                }
            },
            TypeCat::General => quote! {
                if let Ok(φ) = #ty::try_from(α) {
                    #rust_name.push(φ);
                    λv = λ.cdr();
                }
            },
            TypeCat::Unsupported => panic!("Unsupported type category for typed symbol: {:?}", ty),
        };

        quote! {
            if let Some(λ) = λv.as_cons() {
                let α = λ.car();
                #ty_parser
                drop(α);
                drop(λ);
            } else {
                #rust_name = None;
            }
        }
    }

    /// Generate parser variable declarations for this typed symbol.
    fn gen_parser_var_decls(&self, m: FieldMod) -> TokenStream {
        let rust_name = &self.rust_name;
        if rust_name != "_" {
            match m {
                FieldMod::Vectored => quote! { let mut #rust_name = Vec::new(); },
                _ => quote! { let #rust_name; },
            }
        } else {
            quote! {}
        }
    }
    
    /// Generate struct field setters for this typed symbol.
    fn gen_struct_field_setters(&self, m: FieldMod) -> TokenStream {
        let rust_name = &self.rust_name;
        if rust_name != "_" {
            quote! { #rust_name, }
        } else {
            quote! {}
        }
    }
    
    /// Return the field names used for the s-expression representing this typed symbol.
    fn field_names(&self) -> Vec<Ident> {
        if self.rust_name == "_" {
            vec![]
        } else {
            vec![self.sexpr_name.clone()]
        }
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
    use {
        super::*,
        crate::{TypeCat, TypeExt},
        pretty_assertions::assert_eq,
        quote::quote,
        syn::parse2,
    };

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
    fn option_des_list_shape_good() {
        let s: Shape = parse2(quote! { [(hello x => foo: i64 [y => bar: String])]}).unwrap();
        let o = s.option_inner().expect("Expected an option");
        let Shape::DesList(ls) = &o else {
            panic!("Not a list shape: {:?}", o);
        };
        assert_eq!(ls.sexpr_head, "hello");
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
        assert_eq!(i1.ty.category(), TypeCat::String, "Not a string: {:?}", i1.ty);
    }

    #[test]
    fn symbol_flag_good() {
        let s: Shape = parse2(quote! { [hello] }).unwrap();
        let sf = s.as_symbol_flag().expect("Expected a symbol flag");
        assert_eq!(sf.sexpr_name, "hello");
        assert_eq!(sf.rust_name, "hello");
    }
}
