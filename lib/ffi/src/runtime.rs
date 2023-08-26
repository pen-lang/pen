use crate::Error;
use std::sync::RwLock;
use tokio::runtime::Handle;

#[no_mangle]
static _PEN_FFI_RUNTIME_HANDLE: RwLock<Option<tokio::runtime::Handle>> = RwLock::new(None);

pub fn set_handle(handle: Handle) -> Result<(), Error> {
    HANDLE.write()?.replace(handle);

    Ok(())
}

pub fn handle() -> Handle {
    Handle::current()
}
