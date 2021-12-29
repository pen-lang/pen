#[no_mangle]
pub extern "C" fn _pen_spawn(closure: ffi::Closure) -> ffi::Closure {
    closure
}
