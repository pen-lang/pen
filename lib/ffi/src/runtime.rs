use std::sync::RwLock;
use tokio::runtime::Handle;

static HANDLE: RwLock<Option<tokio::runtime::Handle>> = RwLock::new(None);

pub fn set_handle(handle: Handle) -> Result<(), RwLockError> {
    HANDLE.write()?.replace(handle);

    Ok(())
}

pub fn handle() -> Handle {
    Handle::current()
}
