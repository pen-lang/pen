use pen_ffi_macro::bindgen;
use std::process::exit;

#[bindgen(crate = "pen_ffi")]
fn default_return_type() {}

#[bindgen(crate = "pen_ffi")]
async fn async_default_return_type() {}

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

#[bindgen(crate = "pen_ffi")]
fn result_function() -> Result<f64, String> {
    Ok(42.0)
}

#[bindgen(crate = "pen_ffi")]
async fn async_result_function() -> Result<f64, String> {
    Ok(42.0)
}

#[bindgen(crate = "pen_ffi")]
fn none_result_function() -> Result<(), String> {
    Ok(())
}

#[bindgen(crate = "pen_ffi")]
async fn async_none_result_function() -> Result<(), String> {
    Ok(())
}
