use crate::utilities::is_os_debug;
use std::alloc::{Layout, alloc, dealloc, realloc};

#[unsafe(no_mangle)]
pub extern "C" fn _pen_malloc(size: usize) -> *mut u8 {
    let pointer =
        unsafe { alloc(Layout::from_size_align(size, ffi::DEFAULT_MEMORY_ALIGNMENT).unwrap()) };

    if is_os_debug() {
        eprintln!("malloc: {} -> {:x}", size, pointer as usize);
    }

    pointer
}

#[unsafe(no_mangle)]
pub extern "C" fn _pen_realloc(old_pointer: *mut u8, size: usize) -> *mut u8 {
    // Layouts are expected to be ignored by the global allocator.
    let new_pointer = unsafe {
        realloc(
            old_pointer,
            Layout::from_size_align(0, ffi::DEFAULT_MEMORY_ALIGNMENT).unwrap(),
            size,
        )
    };

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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _pen_free(pointer: *mut u8) {
    if is_os_debug() {
        eprintln!("free: {:x}", pointer as usize);
    }

    let layout = Layout::from_size_align(0, ffi::DEFAULT_MEMORY_ALIGNMENT).unwrap();

    unsafe { dealloc(pointer, layout) }
}
