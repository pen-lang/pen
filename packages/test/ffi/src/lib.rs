#![no_std]

extern crate alloc;

use core::mem::transmute;

#[ffi::bindgen]
fn _pen_test_state_new() -> ffi::Number {
    unsafe { transmute(transmute::<_, u64>(x) & transmute::<_, u64>(y)) }
}
