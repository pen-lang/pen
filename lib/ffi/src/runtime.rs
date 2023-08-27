mod error;

use crate::Error;
use error::RuntimeError;
use std::sync::RwLock;
use tokio::runtime::Handle;

#[no_mangle]
#[linkage = "weak"]
static _PEN_FFI_RUNTIME_HANDLE: RwLock<Option<tokio::runtime::Handle>> = RwLock::new(None);

pub fn set_handle(handle: Handle) -> Result<(), Error> {
    _PEN_FFI_RUNTIME_HANDLE
        .write()
        .map_err(|_| RuntimeError::HandleLockPoisoned)?
        .replace(handle);

    Ok(())
}

pub fn handle() -> Result<Handle, Error> {
    let guard = _PEN_FFI_RUNTIME_HANDLE.read()?;

    if let Some(handle) = guard.as_ref() {
        Ok(handle.clone())
    } else {
        Err(RuntimeError::HandleNotInitialized.into())
    }
}
