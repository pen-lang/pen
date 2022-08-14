#[ffi::bindgen]
fn _pen_os_get_arguments() -> ffi::List {
    std::env::args()
        .skip(1)
        .map(ffi::ByteString::from)
        .collect::<Vec<_>>()
        .into()
}
