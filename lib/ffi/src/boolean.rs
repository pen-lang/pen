#[pen_ffi_macro::into_any(crate = "crate", name = "pen_ffi_boolean_to_any")]
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Boolean {
    value: bool,
}

impl Boolean {
    pub fn new(value: bool) -> Self {
        Self { value }
    }
}

impl From<Boolean> for bool {
    fn from(number: Boolean) -> Self {
        number.value
    }
}

impl From<bool> for Boolean {
    fn from(value: bool) -> Self {
        Self::new(value)
    }
}
