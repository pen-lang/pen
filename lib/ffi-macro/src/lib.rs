mod function;
mod into_any;
mod type_;
mod utilities;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn bindgen(attributes: TokenStream, item: TokenStream) -> TokenStream {
    function::generate_binding(attributes, item)
}

#[proc_macro_attribute]
pub fn any(attributes: TokenStream, item: TokenStream) -> TokenStream {
    type_::generate_binding(attributes, item)
}

#[proc_macro_attribute]
pub fn into_any(attributes: TokenStream, item: TokenStream) -> TokenStream {
    into_any::generate(attributes, item)
}
