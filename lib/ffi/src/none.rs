#[pen_ffi_macro::into_any(crate = "crate", fn = "pen_ffi_any_from_none")]
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct None {
    _private: [u8; 0],
}

impl None {
    pub const fn new() -> Self {
        Self { _private: [] }
    }
}
