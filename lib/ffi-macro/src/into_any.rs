use crate::utilities::parse_crate_path;
use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, Ident, ItemStruct};

pub fn generate(attributes: TokenStream, item: TokenStream) -> TokenStream {
    let attributes = parse_macro_input!(attributes as AttributeArgs);
    let type_ = parse_macro_input!(item as ItemStruct);

    match generate_type(&attributes, &type_) {
        Ok(tokens) => tokens,
        Err(message) => quote! { compile_error!(#message) }.into(),
    }
}

fn generate_type(attributes: &AttributeArgs, type_: &ItemStruct) -> Result<TokenStream, String> {
    let crate_path = parse_crate_path(attributes)?;
    let module_name = Ident::new(
        &(type_.ident.to_string().to_case(Case::Snake) + "_module"),
        type_.ident.span(),
    );
    let type_name = &type_.ident;

    Ok(quote! {
        #type_

        mod #module_name {
            use core::{alloc::Layout, mem, ptr};
            use super::#type_name;

            #[test]
            fn type_size() {
                assert!(
                    Layout::new::<#type_name>().size() <= Layout::new::<*const u8>().size(),
                    "type size too large",
                );
            }

            unsafe fn transmute_into_payload<T: Send + Sync>(data: T) -> u64 {
                let mut payload = 0;

                ptr::write(&mut payload as *mut u64 as *mut T, data);

                payload
            }

            unsafe fn transmute_from_payload<T: Send + Sync>(payload: u64) -> T {
                ptr::read(&payload as *const u64 as *const T)
            }

            #[allow(clippy::forget_copy, clippy::forget_non_drop)]
            extern "C" fn clone(x: u64) -> u64 {
                let x = unsafe { transmute_from_payload::<#type_name>(x) };
                let payload = unsafe { transmute_into_payload(x.clone()) };

                mem::forget(x);

                payload
            }

            extern "C" fn drop(x: u64) {
                unsafe { transmute_from_payload::<#type_name>(x) };
            }

            extern "C" fn synchronize(_: u64) {
                // Currently, all types in Rust are expected to implement Sync.
            }

            static TYPE_INFORMATION: #crate_path::TypeInformation =
                #crate_path::TypeInformation { clone, drop, synchronize };

            impl From<#type_name> for #crate_path::Any {
                fn from(x: #type_name) -> Self {
                    Self::new(&TYPE_INFORMATION, unsafe { transmute_into_payload(x) })
                }
            }

            impl TryFrom<#crate_path::Any> for #type_name {
                type Error = ();

                fn try_from(any: #crate_path::Any) -> Result<Self, ()> {
                    if ptr::eq(any.type_information(), &TYPE_INFORMATION) {
                        let x = unsafe { transmute_from_payload(*any.payload()) };
                        mem::forget(any);
                        Ok(x)
                    } else {
                        Err(())
                    }
                }
            }

            impl<'a> TryFrom<&'a #crate_path::Any> for &'a #type_name {
                type Error = ();

                fn try_from(any: &#crate_path::Any) -> Result<Self, ()> {
                    if ptr::eq(any.type_information(), &TYPE_INFORMATION) {
                        Ok(unsafe { mem::transmute(any.payload()) })
                    } else {
                        Err(())
                    }
                }
            }

            impl<'a> TryFrom<&'a mut #crate_path::Any> for &'a mut #type_name {
                type Error = ();

                fn try_from(any: &mut #crate_path::Any) -> Result<Self, ()> {
                    if ptr::eq(any.type_information(), &TYPE_INFORMATION) {
                        Ok(unsafe { mem::transmute(any.payload()) })
                    } else {
                        Err(())
                    }
                }
            }
        }
    }
    .into())
}
