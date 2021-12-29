use std::os::raw::c_void;

#[repr(C)]
#[derive(Clone)]
pub struct Closure {
    entry_pointer: *const c_void,
    drop_function: extern "C" fn(*mut u8),
    arity: usize,
}

impl Closure {
    pub fn new(entry_pointer: *const c_void, arity: usize) -> Self {
        Self {
            entry_pointer,
            drop_function: drop_nothing,
            arity,
        }
    }
}

extern "C" fn drop_nothing(_: *mut u8) {}

impl Drop for Closure {
    fn drop(&mut self) {
        (self.drop_function)(self as *mut Self as *mut u8);
    }
}
