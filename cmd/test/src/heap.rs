use std::{alloc::Layout, os::raw::c_void};

const DEFAULT_ALIGNMENT: usize = 8;

#[no_mangle]
pub extern "C" fn _pen_malloc(size: usize) -> *mut c_void {
    (unsafe { std::alloc::alloc(Layout::from_size_align(size, DEFAULT_ALIGNMENT).unwrap()) })
        as *mut c_void
}

#[no_mangle]
pub extern "C" fn _pen_realloc(old_pointer: *mut c_void, size: usize) -> *mut c_void {
    // Layouts are expected to be ignored by the global allocator.
    (unsafe {
        std::alloc::realloc(
            old_pointer as *mut u8,
            Layout::from_size_align(0, DEFAULT_ALIGNMENT).unwrap(),
            size,
        )
    }) as *mut c_void
}

/// # Safety
///
/// Pointers returned from `_pen_malloc` or `_pen_realloc` must be passed.
#[no_mangle]
pub unsafe extern "C" fn _pen_free(pointer: *mut u8) {
    std::alloc::dealloc(
        pointer,
        Layout::from_size_align(0, DEFAULT_ALIGNMENT).unwrap(),
    )
}
