static mut HANDLE: Option<tokio::runtime::Handle> = None;

pub fn handle() -> tokio::runtime::Handle {
    Handle::current()
}
