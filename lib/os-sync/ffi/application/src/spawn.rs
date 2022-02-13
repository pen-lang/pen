extern "C" {
    fn _pen_os_join(xs: ffi::Arc<ffi::List>) -> ffi::Arc<ffi::List>;
}

#[no_mangle]
extern "C" fn _pen_spawn(closure: ffi::Arc<ffi::Closure>) -> ffi::Arc<ffi::Closure> {
    closure
}

#[no_mangle]
extern "C" fn _pen_join(xs: ffi::Arc<ffi::List>) -> ffi::Arc<ffi::List> {
    unsafe { _pen_os_join(xs) }
}
