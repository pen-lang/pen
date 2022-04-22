use rand::random;

#[ffi::bindgen]
fn _pen_os_random_number() -> ffi::Number {
    random::<f64>().into()
}
