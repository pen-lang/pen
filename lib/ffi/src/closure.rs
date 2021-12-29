use std::{os::raw::c_void, ptr::drop_in_place};

#[repr(C)]
#[derive(Clone)]
pub struct Closure<T = ()> {
    entry_function: *const c_void,
    drop_function: extern "C" fn(&mut Self),
    payload: T,
}

impl<T> Closure<T> {
    pub fn new(entry_function: *const c_void, payload: T) -> Self {
        Self {
            entry_function,
            drop_function,
            payload,
        }
    }

    pub fn entry_function(&self) -> *const c_void {
        self.entry_function
    }

    pub fn payload(&mut self) -> &mut T {
        &mut self.payload
    }
}

extern "C" fn drop_function<T>(closure: &mut Closure<T>) {
    unsafe { drop_in_place(closure.payload()) }
}

impl<T> Drop for Closure<T> {
    fn drop(&mut self) {
        (self.drop_function)(self);
    }
}
