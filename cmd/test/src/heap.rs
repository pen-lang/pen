use std::alloc::Layout;

#[no_mangle]
pub extern "C" fn _pen_malloc(size: usize) -> *mut u8 {
    (unsafe {
        std::alloc::alloc(Layout::from_size_align(size, ffi::DEFAULT_MEMORY_ALIGNMENT).unwrap())
    }) as *mut u8
}

#[no_mangle]
pub extern "C" fn _pen_realloc(old_pointer: *mut u8, size: usize) -> *mut u8 {
    // Layouts are expected to be ignored by the global allocator.
    (unsafe {
        std::alloc::realloc(
            old_pointer as *mut u8,
            Layout::from_size_align(0, ffi::DEFAULT_MEMORY_ALIGNMENT).unwrap(),
            size,
        )
    }) as *mut u8
}

/// # Safety
///
/// Pointers returned from `_pen_malloc` or `_pen_realloc` must be passed.
#[no_mangle]
pub unsafe extern "C" fn _pen_free(pointer: *mut u8) {
    std::alloc::dealloc(
        pointer,
        Layout::from_size_align(0, ffi::DEFAULT_MEMORY_ALIGNMENT).unwrap(),
    )
}
