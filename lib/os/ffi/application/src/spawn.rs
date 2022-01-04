use std::pin::Pin;
use tokio::{spawn, task::JoinHandle};

#[no_mangle]
extern "C" fn _pen_spawn(
    closure: ffi::Arc<ffi::Closure>,
) -> ffi::Arc<ffi::Closure<Option<Pin<Box<JoinHandle<ffi::Any>>>>>> {
    ffi::future::to_closure(spawn(ffi::future::from_closure(closure)))
}
