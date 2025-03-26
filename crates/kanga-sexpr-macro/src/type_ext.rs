use {
    log::warn,
    proc_macro2::{Ident, Span, TokenStream},
    quote::{quote, ToTokens},
    syn::{GenericArgument, PathArguments, PathSegment, Type},
};

/// The general category of a type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TypeCat {
    Float,
    Int,
    String,
    Uuid,
    General,
    Unsupported,
}

/// Extensions for the `Type` enum.
pub(crate) trait TypeExt {
    /// Returns the category of the type.
    fn category(&self) -> TypeCat;

    /// If the type is a numeric type, this returns `Some(&Type)`.
    /// Otherwise, it returns `None`.
    fn as_numeric(&self) -> Option<&Ident>;

    /// Return a string representing the Rust expression for this type.
    fn to_string(&self) -> String;
}

impl TypeExt for Type {
    fn category(&self) -> TypeCat {
        let Type::Path(path) = self else {
            warn!("Type is not a path: {:?}", self);
            return TypeCat::Unsupported;
        };

        if path.qself.is_some() {
            warn!("Type has a qself: {:?}", self);
            return TypeCat::Unsupported;
        }

        let segments = &path.path.segments;
        let n_segments = segments.len();

        if n_segments == 0 {
            warn!("Type has no segments: {:?}", self);
            return TypeCat::Unsupported;
        }

        let seg0 = segments[0].ident.to_string();
        if n_segments == 1 {
            match seg0.as_str() {
                "f64" => return TypeCat::Float,
                "i64" => return TypeCat::Int,
                "String" => return TypeCat::String,
                "Uuid" => return TypeCat::Uuid,
                _ => return TypeCat::General,
            }
        }

        let seg1 = &segments[1].ident.to_string();
        if n_segments == 2 {
            if seg0 == "uuid" && seg1 == "Uuid" {
                return TypeCat::Uuid;
            } else {
                return TypeCat::General;
            }
        }

        let seg2 = &segments[2].ident.to_string();
        if n_segments == 3 {
            if seg0 == "std" && seg1 == "string" && seg2 == "String" {
                return TypeCat::String;
            } else {
                return TypeCat::General;
            }
        }

        TypeCat::General
    }

    fn as_numeric(&self) -> Option<&Ident> {
        const NUMERIC_TYPES: &[&str] =
            &["i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32", "f64"];

        let Type::Path(path) = self else {
            return None;
        };

        if path.qself.is_some() {
            return None;
        }

        let segments = &path.path.segments;
        if segments.len() != 1 {
            return None;
        }

        let seg0 = &segments[0];
        let seg0_ident = &seg0.ident;

        for ty in NUMERIC_TYPES {
            if seg0_ident == ty {
                return Some(seg0_ident);
            }
        }

        None
    }

    fn to_string(&self) -> String {
        self.to_token_stream().to_string()
    }
}
