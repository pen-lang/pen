use pen_ffi::{ByteString, Number};
use pen_ffi_macro::{bindgen, runtime};

#[runtime(crate = "pen_ffi")]
async fn default_return_type() {}

#[runtime(crate = "pen_ffi")]
async fn function() -> f64 {
    42.0
}

#[runtime(crate = "pen_ffi")]
async fn result_function() -> Result<Number, ByteString> {
    let x = Ok::<_, ByteString>(42.0.into())?;

    Ok(x)
}

#[runtime(crate = "pen_ffi")]
async fn argument_function(x: f64, y: f64) -> f64 {
    x + y
}

#[runtime(crate = "pen_ffi")]
async fn mut_argument_function(mut x: f64) -> f64 {
    x += 42.0;
    x
}

#[runtime(crate = "pen_ffi")]
pub async fn public_function() -> f64 {
    42.0
}

#[runtime(crate = "pen_ffi")]
#[allow(unreachable_code)]
async fn unreachable_by_exit() {
    unreachable!()
}

#[bindgen(crate = "pen_ffi")]
#[runtime(crate = "pen_ffi")]
async fn bindgen_default_return_type() {}

#[bindgen(crate = "pen_ffi")]
#[runtime(crate = "pen_ffi")]
async fn bindgen_function() -> f64 {
    42.0
}

#[bindgen(crate = "pen_ffi")]
#[runtime(crate = "pen_ffi")]
async fn bindgen_result_function(x: Number) -> Result<Number, ByteString> {
    Ok(x)
}

#[runtime(crate = "pen_ffi")]
#[bindgen(crate = "pen_ffi")]
async fn runtime_bindgen_result_function(x: Number) -> Result<Number, ByteString> {
    Ok(x)
}
