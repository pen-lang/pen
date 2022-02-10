use pen_ffi_macro::bindgen;
use std::process::exit;

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

#[bindgen(crate = "pen_ffi")]
#[allow(unreachable_code)]
fn unreachable_by_exit() {
    exit(0)
}
