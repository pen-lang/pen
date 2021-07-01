use std::alloc::{alloc, Layout};

const DEFAULT_ALIGNMENT: usize = 8;

#[repr(C)]
pub struct Stack {
    base_pointer: *mut u8,
    size: usize,
    capacity: usize,
}

impl Stack {
    pub fn new(capacity: usize) -> Self {
        Self {
            base_pointer: unsafe {
                alloc(Layout::from_size_align(capacity, DEFAULT_ALIGNMENT).unwrap())
            },
            size: 0,
            capacity,
        }
    }
}
