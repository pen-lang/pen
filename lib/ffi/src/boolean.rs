#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Boolean {
    value: usize,
}

impl Boolean {
    pub fn new(value: bool) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl From<Boolean> for usize {
    fn from(number: Boolean) -> Self {
        number.value
    }
}

impl From<bool> for Boolean {
    fn from(value: bool) -> Self {
        Self::new(value)
    }
}
