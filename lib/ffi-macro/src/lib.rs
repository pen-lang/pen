mod any;
mod bindgen;
mod into_any;
mod utilities;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn bindgen(attributes: TokenStream, item: TokenStream) -> TokenStream {
    bindgen::generate(attributes, item)
}

#[proc_macro_attribute]
pub fn any(attributes: TokenStream, item: TokenStream) -> TokenStream {
    any::generate(attributes, item)
}

#[proc_macro_attribute]
pub fn into_any(attributes: TokenStream, item: TokenStream) -> TokenStream {
    into_any::generate(attributes, item)
}
