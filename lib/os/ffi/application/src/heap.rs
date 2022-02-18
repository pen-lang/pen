use crate::utilities::is_os_debug;
use std::alloc::{alloc, dealloc, realloc, Layout};

const MAX_STACK_SIZE: usize = 2 << (2 * 10 + 8);

#[no_mangle]
pub extern "C" fn _pen_malloc(size: usize) -> *mut u8 {
    check_stack_size(size);

    let pointer =
        (unsafe { alloc(Layout::from_size_align(size, ffi::DEFAULT_MEMORY_ALIGNMENT).unwrap()) })
            as *mut u8;

    if is_os_debug() {
        eprintln!("malloc: {} -> {:x}", size, pointer as usize);
    }

    pointer
}

#[no_mangle]
pub extern "C" fn _pen_realloc(old_pointer: *mut u8, size: usize) -> *mut u8 {
    check_stack_size(size);

    // Layouts are expected to be ignored by the global allocator.
    let new_pointer = (unsafe {
        realloc(
            old_pointer as *mut u8,
            Layout::from_size_align(0, ffi::DEFAULT_MEMORY_ALIGNMENT).unwrap(),
            size,
        )
    }) as *mut u8;

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

    dealloc(
        pointer,
        Layout::from_size_align(0, ffi::DEFAULT_MEMORY_ALIGNMENT).unwrap(),
    )
}

fn check_stack_size(size: usize) {
    if size > MAX_STACK_SIZE {
        panic!("");
    }
}
