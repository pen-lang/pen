use pen_ffi_macro::bindgen;

#[bindgen(crate = "pen_ffi")]
fn default_return_type() {}

#[bindgen(crate = "pen_ffi")]
fn async_default_return_type() {}

#[bindgen]
fn sync_function() -> f64 {
    42.0
}

#[bindgen(crate = "pen_ffi")]
async fn async_function() -> f64 {
    42.0
}
