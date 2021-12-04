use crate::result::FfiResult;

#[no_mangle]
extern "C" fn _pen_os_read_directory(
    _path: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::Arc<ffi::extra::StringArray>>> {
    todo!()
}

#[no_mangle]
extern "C" fn _pen_os_create_directory(_path: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::None>> {
    todo!()
}

#[no_mangle]
extern "C" fn _pen_os_remove_directory(_path: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::None>> {
    todo!()
}
