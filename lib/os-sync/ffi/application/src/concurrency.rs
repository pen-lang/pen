#[ffi::bindgen]
fn _pen_spawn(closure: ffi::Arc<ffi::Closure>) -> ffi::Arc<ffi::Closure> {
    closure
}
