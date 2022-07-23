#[ffi::bindgen]
fn _pen_spawn(closure: ffi::Arc<ffi::Closure>) -> ffi::Arc<ffi::Closure> {
    closure
}

#[ffi::bindgen]
async fn _pen_yield() -> ffi::None {
    unreachable!("thunk lock detected")
}
