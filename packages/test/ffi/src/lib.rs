#![no_std]

extern crate alloc;

use core::mem::transmute;

#[ffi::bindgen]
fn _pen_core_bit_and(x: ffi::Number, y: ffi::Number) -> ffi::Number {
    unsafe { transmute(transmute::<_, u64>(x) & transmute::<_, u64>(y)) }
}
