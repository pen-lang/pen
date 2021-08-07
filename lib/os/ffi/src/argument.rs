use super::array::Array;

#[no_mangle]
extern "C" fn _pen_os_get_arguments() -> Array {
    std::env::args()
        .map(|string| ffi::ByteString::from(string))
        .collect::<Vec<ffi::ByteString>>()
        .into()
}
