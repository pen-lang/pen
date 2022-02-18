use std::alloc::{alloc, dealloc, realloc, Layout};

const MAX_STACK_SIZE: usize = 2 << (2 * 10);

#[no_mangle]
pub extern "C" fn _pen_malloc(size: usize) -> *mut u8 {
    check_stack_size(size);

    (unsafe { alloc(Layout::from_size_align(size, ffi::DEFAULT_MEMORY_ALIGNMENT).unwrap()) })
        as *mut u8
}

#[no_mangle]
pub extern "C" fn _pen_realloc(old_pointer: *mut u8, size: usize) -> *mut u8 {
    check_stack_size(size);

    // Layouts are expected to be ignored by the global allocator.
    (unsafe {
        realloc(
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
    dealloc(
        pointer,
        Layout::from_size_align(0, ffi::DEFAULT_MEMORY_ALIGNMENT).unwrap(),
    )
}

fn check_stack_size(size: usize) {
    if size > MAX_STACK_SIZE {
        panic!(
            "stack overflow: {} bytes (max: {} bytes)",
            size, MAX_STACK_SIZE
        );
    }
}
