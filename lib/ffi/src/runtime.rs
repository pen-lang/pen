use tokio::runtime::Handle;

static mut HANDLE: Option<tokio::runtime::Handle> = None;

pub fn set_handle(handle: Handle) {
    HANDLE = Some(handle)
}

pub fn handle() -> Handle {
    Handle::current()
}
