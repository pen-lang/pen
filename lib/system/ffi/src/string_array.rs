#[no_mangle]
extern "C" fn _pen_system_string_array_get(
    array: ffi::Arc<ffi::extra::StringArray>,
    index: ffi::Number,
) -> ffi::ByteString {
    array.get(f64::from(index) as usize - 1).unwrap_or_default()
}

#[no_mangle]
extern "C" fn _pen_system_string_array_length(
    array: ffi::Arc<ffi::extra::StringArray>,
) -> ffi::Number {
    (array.len() as f64).into()
}
