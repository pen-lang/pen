mod function;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn bindgen(attributes: TokenStream, item: TokenStream) -> TokenStream {
    function::generate_binding(attributes, item)
}
