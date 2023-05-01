use crate::utilities::{generate_type_size_test, parse_crate_path};
use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use std::error::Error;
use syn::{Attribute, Ident, ItemStruct};

pub fn generate(
    attributes: &[Attribute],
    type_: &ItemStruct,
) -> Result<TokenStream, Box<dyn Error>> {
    let crate_path = parse_crate_path(attributes)?;
    let module_name = Ident::new(
        &(type_.ident.to_string().to_case(Case::Snake) + "_module"),
        type_.ident.span(),
    );
    let type_name = &type_.ident;
    let type_size_test = generate_type_size_test(type_name);

    Ok(quote! {
        #type_

        mod #module_name {
            use core::{alloc::Layout, mem, ptr};
            use super::#type_name;

            #type_size_test

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
                #crate_path::TypeInformation::new(clone, drop, synchronize);

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
