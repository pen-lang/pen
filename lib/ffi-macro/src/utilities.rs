use crate::attribute_list::AttributeList;
use quote::quote;
use std::error::Error;
use syn::{Expr, ExprLit, Ident, Lit, Meta, Path, parse::Parse, parse_str};

const DEFAULT_CRATE_NAME: &str = "ffi";

pub fn parse_crate_path(attributes: &AttributeList) -> Result<Path, Box<dyn Error>> {
    Ok(parse_string_attribute(attributes, "crate")?.unwrap_or(parse_str(DEFAULT_CRATE_NAME)?))
}

pub fn parse_string_attribute<T: Parse>(
    attributes: &AttributeList,
    key: &str,
) -> Result<Option<T>, Box<dyn Error>> {
    Ok(attributes
        .variables()
        .find_map(|meta| match meta {
            Meta::NameValue(name_value) => {
                if name_value.path.is_ident(key) {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(string),
                        ..
                    }) = &name_value.value
                    {
                        Some(string.value())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        })
        .map(|string| parse_str(&string))
        .transpose()?)
}

pub fn generate_type_size_test(type_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        #[test]
        fn type_size() {
            use core::alloc::Layout;

            assert!(
                Layout::new::<#type_name>().size() <= Layout::new::<*const u8>().size(),
                "type size too large",
            );
        }
    }
}
