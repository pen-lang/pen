#[no_mangle]
pub extern "C" fn _pen_spawn(closure: ffi::Arc<ffi::Closure>) -> ffi::Arc<ffi::Closure> {
    closure
}
