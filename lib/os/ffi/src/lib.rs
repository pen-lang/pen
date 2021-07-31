mod file;
mod open_file_options;
mod result;

use result::FfiResult;
#[cfg(not(test))]
use std::os::raw::c_int;
use std::{
    alloc::Layout,
    fs::File,
    io::{Read, Write},
    os::{raw::c_void, unix::io::FromRawFd},
};

const DEBUG_ENVIRONMENT_VARIABLE: &str = "PEN_DEBUG";
const DEFAULT_ALIGNMENT: usize = 8;
#[cfg(not(test))]
const INITIAL_STACK_CAPACITY: usize = 256;

extern "C" {
    fn _pen_os_main(
        stack: *mut ffi::cps::Stack,
        continuation: extern "C" fn(*mut ffi::cps::Stack, f64) -> ffi::cps::Result,
        argument: ffi::None,
    ) -> ffi::cps::Result;
}

#[no_mangle]
pub extern "C" fn _pen_malloc(size: usize) -> *mut c_void {
    let pointer =
        (unsafe { std::alloc::alloc(Layout::from_size_align(size, DEFAULT_ALIGNMENT).unwrap()) })
            as *mut c_void;

    if std::env::var(DEBUG_ENVIRONMENT_VARIABLE).is_ok() {
        eprintln!("malloc: {} -> {:x}", size, pointer as usize);
    }

    pointer
}

#[no_mangle]
pub extern "C" fn _pen_realloc(old_pointer: *mut c_void, size: usize) -> *mut c_void {
    // Layouts are expected to be ignored by the global allocator.
    let new_pointer = (unsafe {
        std::alloc::realloc(
            old_pointer as *mut u8,
            Layout::from_size_align(0, DEFAULT_ALIGNMENT).unwrap(),
            size,
        )
    }) as *mut c_void;

    if std::env::var(DEBUG_ENVIRONMENT_VARIABLE).is_ok() {
        eprintln!(
            "realloc: {:x}, {} -> {:x}",
            old_pointer as usize, size, new_pointer as usize
        );
    }

    new_pointer
}

/// # Safety
///
/// Pointers returned from `_pen_malloc` or `_pen_realloc` must be passed.
#[no_mangle]
pub unsafe extern "C" fn _pen_free(pointer: *mut u8) {
    if std::env::var(DEBUG_ENVIRONMENT_VARIABLE).is_ok() {
        eprintln!("free: {:x}", pointer as usize);
    }

    std::alloc::dealloc(
        pointer,
        Layout::from_size_align(0, DEFAULT_ALIGNMENT).unwrap(),
    )
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() -> c_int {
    let mut stack = ffi::cps::Stack::new(INITIAL_STACK_CAPACITY);

    unsafe { _pen_os_main(&mut stack, exit, ffi::None::new()) };

    unreachable!()
}

#[cfg(not(test))]
extern "C" fn exit(_: *mut ffi::cps::Stack, code: f64) -> ffi::cps::Result {
    std::process::exit(code as i32)
}

#[no_mangle]
extern "C" fn _pen_os_fd_read(
    fd: ffi::Number,
    buffer_size: ffi::Number,
) -> ffi::Arc<FfiResult<ffi::ByteString>> {
    let mut file = unsafe { File::from_raw_fd(f64::from(fd) as i32) };
    let mut buffer = vec![0; f64::from(buffer_size) as usize];

    let count = match file.read(&mut buffer) {
        Ok(count) => count,
        Err(error) => return ffi::Arc::new(error.into()),
    };
    buffer.resize(count, 0);

    std::mem::forget(file);

    FfiResult::ok(ffi::ByteString::from(buffer)).into()
}

#[no_mangle]
extern "C" fn _pen_os_fd_write(
    fd: ffi::Number,
    buffer: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::Number>> {
    let mut file = unsafe { File::from_raw_fd(f64::from(fd) as i32) };

    let count = match file.write(buffer.as_slice()) {
        Ok(count) => count,
        Err(error) => return ffi::Arc::new(error.into()),
    };

    std::mem::forget(file);

    FfiResult::ok((count as f64).into()).into()
}
