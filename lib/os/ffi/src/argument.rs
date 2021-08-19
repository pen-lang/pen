use super::string_array::StringArray;
use ffi::AnyLike;

#[no_mangle]
extern "C" fn _pen_os_get_arguments() -> ffi::Arc<StringArray> {
    ffi::Arc::new(
        std::env::args()
            .skip(1)
            .map(ffi::ByteString::from)
            .collect::<Vec<_>>()
            .into(),
    )
}

#[no_mangle]
extern "C" fn _pen_ffi_any_to_string(any: ffi::Any) -> ffi::ByteString {
    ffi::ByteString::from_any(any).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_arguments() {
        _pen_os_get_arguments();
    }
}
