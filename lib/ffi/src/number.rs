#[repr(C)]
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

impl From<f64> for Number {
    fn from(value: f64) -> Self {
        Self { value }
    }
}
