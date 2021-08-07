use super::array::Array;
use ffi::AnyLike;

#[no_mangle]
extern "C" fn _pen_os_get_arguments() -> ffi::Arc<Array> {
    ffi::Arc::new(
        std::env::args()
            .skip(1)
            .map(|string| ffi::ByteString::from(string))
            .collect::<Vec<_>>()
            .into(),
    )
}

#[no_mangle]
extern "C" fn _pen_ffi_any_to_string(any: ffi::Any) -> ffi::ByteString {
    ffi::ByteString::from_any(any).unwrap_or_else(|| ffi::ByteString::from(""))
}
