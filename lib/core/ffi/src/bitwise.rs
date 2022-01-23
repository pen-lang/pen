use std::intrinsics::transmute;

#[ffi::bindgen]
fn _pen_core_bitwise_and(x: ffi::Number, y: ffi::Number) -> ffi::Number {
    unsafe { transmute(transmute::<_, u64>(x) & transmute::<_, u64>(y)) }
}

#[ffi::bindgen]
fn _pen_core_bitwise_or(x: ffi::Number, y: ffi::Number) -> ffi::Number {
    unsafe { transmute(transmute::<_, u64>(x) | transmute::<_, u64>(y)) }
}

#[ffi::bindgen]
fn _pen_core_bitwise_xor(x: ffi::Number, y: ffi::Number) -> ffi::Number {
    unsafe { transmute(transmute::<_, u64>(x) ^ transmute::<_, u64>(y)) }
}

#[ffi::bindgen]
fn _pen_core_bitwise_not(x: ffi::Number) -> ffi::Number {
    unsafe { transmute(!transmute::<_, u64>(x)) }
}

#[ffi::bindgen]
fn _pen_core_bitwise_left_shift(x: ffi::Number, count: ffi::Number) -> ffi::Number {
    unsafe { transmute(transmute::<_, u64>(x) << (f64::from(count) as u64)) }
}

#[ffi::bindgen]
fn _pen_core_bitwise_right_shift(x: ffi::Number, count: ffi::Number) -> ffi::Number {
    unsafe { transmute(transmute::<_, u64>(x) >> (f64::from(count) as u64)) }
}

#[ffi::bindgen]
fn _pen_core_bitwise_integer_64(x: ffi::Number) -> ffi::Number {
    unsafe { transmute(f64::from(x) as u64) }
}
