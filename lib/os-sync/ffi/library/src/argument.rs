#[ffi::bindgen]
fn _pen_os_get_arguments() -> ffi::Arc<ffi::extra::StringArray> {
    ffi::Arc::new(
        std::env::args()
            .skip(1)
            .map(ffi::ByteString::from)
            .collect::<Vec<_>>()
            .into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_arguments() {
        _pen_os_get_arguments();
    }
}
