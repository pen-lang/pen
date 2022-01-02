#[pen_ffi_macro::any(crate = "crate")]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Number {
    value: f64,
}

impl Number {
    pub const fn new(value: f64) -> Self {
        Self { value }
    }
}

impl From<Number> for f64 {
    fn from(number: Number) -> Self {
        number.value
    }
}

impl From<Number> for usize {
    fn from(number: Number) -> Self {
        number.value as Self
    }
}

impl From<f64> for Number {
    fn from(value: f64) -> Self {
        Self { value }
    }
}
