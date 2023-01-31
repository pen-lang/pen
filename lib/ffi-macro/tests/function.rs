use pen_ffi::{ByteString, Number};
use pen_ffi_macro::bindgen;

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
    unreachable!()
}

#[bindgen(crate = "pen_ffi")]
fn result_function() -> Result<Number, ByteString> {
    Ok(42.0.into())
}

#[bindgen(crate = "pen_ffi")]
async fn async_result_function() -> Result<Number, ByteString> {
    Ok(42.0.into())
}

#[bindgen(crate = "pen_ffi")]
fn none_result_function() -> Result<(), ByteString> {
    Ok(())
}

#[bindgen(crate = "pen_ffi")]
async fn async_none_result_function() -> Result<(), ByteString> {
    Ok(())
}

#[bindgen(crate = "pen_ffi")]
fn mut_argument_function(mut x: f64) -> f64 {
    x += 42.0;
    x
}

#[bindgen(crate = "pen_ffi")]
async fn async_mut_argument_function(mut x: f64) -> f64 {
    x += 42.0;
    x
}

#[bindgen(crate = "pen_ffi")]
fn mut_argument_none_function(mut x: f64) {
    x += 42.0;

    println!("{x}");
}

#[bindgen(crate = "pen_ffi")]
async fn async_mut_argument_none_function(mut x: f64) {
    x += 42.0;

    println!("{x}");
}
