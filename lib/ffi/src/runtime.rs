use std::sync::RwLock;
use tokio::runtime::Handle;

static HANDLE: RwLock<Option<tokio::runtime::Handle>> = RwLock::new(None);

pub fn set_handle(handle: Handle) {
    HANDLE.write().unwrap().replace(handle);
}

pub fn handle() -> Handle {
    Handle::current()
}
