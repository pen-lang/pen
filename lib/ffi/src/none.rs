#[pen_ffi_macro::any(crate = "crate")]
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct None {
    _private: [u8; 0],
}

impl None {
    pub const fn new() -> Self {
        Self { _private: [] }
    }
}
