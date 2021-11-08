use crate::utilities::is_os_debug;
use std::{alloc::Layout, os::raw::c_void};

const DEFAULT_ALIGNMENT: usize = 8;

#[no_mangle]
pub extern "C" fn _pen_malloc(size: usize) -> *mut c_void {
    let pointer =
        (unsafe { std::alloc::alloc(Layout::from_size_align(size, DEFAULT_ALIGNMENT).unwrap()) })
            as *mut c_void;

    if is_os_debug() {
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

    if is_os_debug() {
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
    if is_os_debug() {
        eprintln!("free: {:x}", pointer as usize);
    }

    std::alloc::dealloc(
        pointer,
        Layout::from_size_align(0, DEFAULT_ALIGNMENT).unwrap(),
    )
}
