extern "C" {
    fn _pen_os_join(xs: ffi::Arc<ffi::List>) -> ffi::Arc<ffi::List>;
}

#[ffi::bindgen]
fn _pen_spawn(closure: ffi::Arc<ffi::Closure>) -> ffi::Arc<ffi::Closure> {
    closure
}

#[ffi::bindgen]
fn _pen_join(xs: ffi::Arc<ffi::List>) -> ffi::Arc<ffi::List> {
    unsafe { _pen_os_join(xs) }
}

#[ffi::bindgen]
async fn _pen_yield() -> ffi::None {
    unreachable!("thunk lock detected")
}
