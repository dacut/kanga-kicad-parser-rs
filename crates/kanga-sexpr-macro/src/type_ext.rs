use {
    proc_macro2::{Ident, Span, TokenStream},
    quote::{quote, ToTokens},
    syn::{GenericArgument, PathArguments, PathSegment, Type},
};

/// Extensions for the `Type` enum.
pub(crate) trait TypeExt {
    /// If the type is a numeric type, this returns `Some(&Type)`.
    /// Otherwise, it returns `None`.
    fn as_numeric(&self) -> Option<&Ident>;

    /// Generate a Rust type tokens to define this type as a field in a struct.
    fn gen_field_type(&self) -> TokenStream;

    /// Generate a check to verify this field has been seen.
    fn gen_try_from_check(
        &self,
        struct_sexpr_name: &Ident,
        field_sexpr_name: &Ident,
        field_rust_name: &Ident,
    ) -> TokenStream;

    /// Generate a `let <field_name> = None/Vec::new()` statement, as appropritate.
    fn gen_try_from_decl(&self, field_name: &Ident) -> TokenStream {
        if self.is_vec() {
            quote! { let mut #field_name = Vec::new(); }
        } else {
            quote! { let mut #field_name = None; }
        }
    }

    /// Generate a match arm to parse a named field.
    fn gen_try_from_match_arm(
        &self,
        struct_sexpr_name: &Ident,
        field_sexpr_name: &Ident,
        field_rust_name: &Ident,
    ) -> TokenStream;

    /// Generate a parser to parse a positional field.
    fn gen_try_from_pos_parser(&self) -> TokenStream;

    /// Generate a parser for this type from a value cons structure with the given name or, if a
    /// struct, the struct cons with the given name.
    fn gen_parser(&self, value_cons: &Ident, struct_cons: &Ident) -> TokenStream;

    /// Indicates whether the type is a numeric type.
    fn is_numeric(&self) -> bool {
        self.as_numeric().is_some()
    }

    /// Indicates whether the type is a `String`.
    fn is_string(&self) -> bool;

    /// Indicates whether the type is an `Option<T>`.
    fn is_option(&self) -> bool {
        self.option_inner().is_some()
    }

    /// Indicates whether the type is a UUID.
    #[allow(unused)]
    fn is_uuid(&self) -> bool;

    /// Indicates whether the type is a vector.
    fn is_vec(&self) -> bool {
        self.vec_inner().is_some()
    }

    /// If the type is an Option<inner>, this returns `Some(inner)`.
    /// Otherwise, it returns `None`.
    fn option_inner(&self) -> Option<&Type>;

    /// If the type is a Vec<inner>, this returns `Some(inner)`.
    /// Otherwise, it returns `None`.
    fn vec_inner(&self) -> Option<&Type>;

    /// Return a string representing the Rust expression for this type.
    fn to_string(&self) -> String;
}

impl TypeExt for Type {
    fn gen_field_type(&self) -> TokenStream {
        quote! { #self }
    }

    fn gen_try_from_check(
        &self,
        struct_sexpr_name: &Ident,
        field_sexpr_name: &Ident,
        field_rust_name: &Ident,
    ) -> TokenStream {
        if self.is_option() || self.is_vec() {
            quote! {}
        } else {
            quote! {
                let Some(#field_rust_name) = #field_sexpr_name else {
                    return Err(::kanga_kicad_sexpr::ParseError::MissingField(stringify!(#struct_sexpr_name).to_string(), stringify!(#field_sexpr_name).to_string(), ::lexpr::Value::Cons(cons.clone())));
                };
            }
        }
    }

    fn gen_try_from_match_arm(
        &self,
        struct_name: &Ident,
        field_sexpr_name: &Ident,
        field_rust_name: &Ident,
    ) -> TokenStream {
        let (ty, add) = if let Some(vec_ty) = self.vec_inner() {
            (
                vec_ty,
                quote! {
                    #field_rust_name.push(value);
                },
            )
        } else {
            (
                self,
                quote! {
                    if #field_rust_name.is_some() {
                        return Err(::kanga_kicad_sexpr::ParseError::DuplicateField(stringify!(#struct_name).to_string(), stringify!(#field_sexpr_name).to_string(), ::lexpr::Value::Cons(cons.clone())));
                    }

                    #field_rust_name = Some(value);
                },
            )
        };

        let inner = ty.gen_parser(&Ident::new("e_cdr", Span::call_site()), &Ident::new("element", Span::call_site()));

        quote! {
            stringify!(#field_sexpr_name) => {
                let (value, e_cdr) = #inner;
                if !e_cdr.is_null() {
                    return Err(::kanga_kicad_sexpr::ParseError::Unexpected(e_cdr.clone()));
                }

                #add
            }
        }
    }

    fn gen_try_from_pos_parser(&self) -> TokenStream {
        let rest = Ident::new("rest", Span::call_site());
        self.gen_parser(&rest, &rest)
    }

    fn gen_parser(&self, value_cons: &Ident, struct_cons: &Ident) -> TokenStream {
        if let Some(inner_ty) = self.option_inner() {
            let inner_parse = inner_ty.gen_parser(value_cons, struct_cons);
            quote! {
                if !#value_cons.is_null() {
                    let (value, next) = #inner_parse;
                    (Some(value), next)
                } else {
                    (None, #value_cons)
                }
            }
        } else if self.is_string() {
            quote! {
                #value_cons.expect_cons_with_any_str_head()?.to_string()
            }
        } else if let Some(numeric_type) = self.as_numeric() {
            let expect_func = Ident::new(&format!("expect_cons_with_any_{numeric_type}_head"), Span::call_site());

            quote! {
                #value_cons.#expect_func()?;
            }
        } else {
            quote! {
                {
                    (#self::try_from(#struct_cons)?, ::lexpr::Value::Null)
                }
            }
        }
    }

    fn is_string(&self) -> bool {
        let Type::Path(path) = self else {
            return false;
        };

        let segments = &path.path.segments;
        let n_segments = segments.len();

        path.qself.is_none()
            && (n_segments == 3
                && (
                    // This must be ::std::string::String or std::string::String
                    segments[0].ident == "std"
                        && segments[0].arguments.is_none()
                        && segments[1].ident == "string"
                        && segments[1].arguments.is_none()
                        && segments[2].ident == "String"
                        && segments[2].arguments.is_empty()
                )
                || n_segments == 1
                    && (
                        // This must be String
                        segments[0].ident == "String" && segments[0].arguments.is_empty()
                    ))
    }

    fn is_uuid(&self) -> bool {
        let Type::Path(path) = self else {
            return false;
        };

        let segments = &path.path.segments;
        let n_segments = segments.len();

        path.qself.is_none()
            && (n_segments == 2
                && (
                    // This must be ::uuid::Uuid or uuid::Uuid
                    segments[0].ident == "uuid"
                        && segments[0].arguments.is_none()
                        && segments[1].ident == "Uuid"
                        && segments[1].arguments.is_none()
                )
                || n_segments == 1
                    && (
                        // This must be Uuid
                        segments[0].ident == "Uuid" && segments[0].arguments.is_empty()
                    ))
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

    fn option_inner(&self) -> Option<&Type> {
        let Type::Path(path) = self else {
            return None;
        };

        let segments = &path.path.segments;
        let n_segments = segments.len();

        if path.qself.is_none() {
            if n_segments == 3
                && (
                    // This must be ::std::option::Option<inner> or std::option::Option<inner>
                    segments[0].ident == "std"
                        && segments[0].arguments.is_none()
                        && segments[1].ident == "option"
                        && segments[1].arguments.is_none()
                )
            {
                extract_option_from_final_segment(&segments[2])
            } else if n_segments == 1 {
                // This must be Option<inner>
                extract_option_from_final_segment(&segments[0])
            } else {
                None
            }
        } else {
            None
        }
    }

    fn vec_inner(&self) -> Option<&Type> {
        let Type::Path(path) = self else {
            return None;
        };

        let segments = &path.path.segments;
        let n_segments = segments.len();

        if path.qself.is_none() {
            if n_segments == 3
                && (
                    // This must be ::std::vec::Vec<inner> or std::vec::Vec<inner>
                    segments[0].ident == "std"
                        && segments[0].arguments.is_none()
                        && segments[1].ident == "vec"
                        && segments[1].arguments.is_none()
                )
            {
                extract_vec_from_final_segment(&segments[2])
            } else if n_segments == 1 {
                // This must be Vec<inner>
                extract_vec_from_final_segment(&segments[0])
            } else {
                None
            }
        } else {
            None
        }
    }

    fn to_string(&self) -> String {
        self.to_token_stream().to_string()
    }
}

fn extract_vec_from_final_segment(segment: &PathSegment) -> Option<&Type> {
    if segment.ident != "Vec" {
        None
    } else {
        extract_segment_angle_bracket_inner(segment)
    }
}

fn extract_option_from_final_segment(segment: &PathSegment) -> Option<&Type> {
    if segment.ident != "Option" {
        None
    } else {
        extract_segment_angle_bracket_inner(segment)
    }
}

fn extract_segment_angle_bracket_inner(segment: &PathSegment) -> Option<&Type> {
    let PathArguments::AngleBracketed(generics) = &segment.arguments else {
        return None;
    };

    if generics.args.len() != 1 {
        return None;
    }

    let GenericArgument::Type(ty) = &generics.args[0] else {
        return None;
    };

    Some(ty)
}
