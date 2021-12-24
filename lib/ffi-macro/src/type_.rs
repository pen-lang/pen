use crate::utilities::parse_crate_path;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemStruct};

pub fn generate_binding(attributes: TokenStream, item: TokenStream) -> TokenStream {
    let attributes = parse_macro_input!(attributes as AttributeArgs);
    let type_ = parse_macro_input!(item as ItemStruct);

    match generate_type(&attributes, &type_) {
        Ok(tokens) => tokens,
        Err(message) => quote! { compile_error!(#message) }.into(),
    }
}

fn generate_type(attributes: &AttributeArgs, type_: &ItemStruct) -> Result<TokenStream, String> {
    let crate_path = parse_crate_path(attributes)?;
    let type_name = &type_.ident;

    Ok(quote! {
        #type_

        mod #type_name {
            unsafe fn transmute_into_payload<T>(data: T) -> u64 {
                let mut payload = 0;

                std::ptr::write(&mut payload as *mut u64 as *mut T, data);

                payload
            }

            unsafe fn transmute_from_payload<T>(payload: u64) -> T {
                std::ptr::read(&payload as *const u64 as *const T)
            }

            #[allow(clippy::forget_copy)]
            extern "C" fn clone(x: u64) -> u64 {
                let x = unsafe { transmute_from_payload::<#type_name>(x) };
                let payload = unsafe { transmute_into_payload(x.clone()) };

                std::mem::forget(x);

                payload
            }

            extern "C" fn drop(x: u64) {
                unsafe { transmute_from_payload::<#type_name>(x) };
            }

            static TYPE_INFORMATION: #crate_path::TypeInformation =
                #crate_path::TypeInformation { clone, drop };

            impl From<#type_name> for #crate_path::Any {
                fn from(x: #type_name) -> Self {
                    Self::new(&TYPE_INFORMATION, unsafe { transmute_into_payload(x) })
                }
            }

            impl TryFrom<#crate_path::Any> for #type_name {
                type Error = ();

                fn try_from(any: #crate_path::Any) -> Result<Self, ()> {
                    if std::ptr::eq(any.type_information(), &TYPE_INFORMATION) {
                        let x = unsafe { transmute_from_payload(*any.payload()) };
                        std::mem::forget(any);
                        Ok(x)
                    } else {
                        Err(())
                    }
                }
            }

            impl<'a> TryFrom<&'a #crate_path::Any> for &'a #type_name {
                type Error = ();

                fn try_from(any: &#crate_path::Any) -> Result<Self, ()> {
                    if std::ptr::eq(any.type_information(), &TYPE_INFORMATION) {
                        Ok(unsafe { std::mem::transmute(any.payload()) })
                    } else {
                        Err(())
                    }
                }
            }
        }
    }
    .into())
}
