#[repr(C)]
#[derive(Clone, Default)]
pub struct Result {
    _private: [u8; 0],
}

impl Result {
    pub fn new() -> Self {
        Self { _private: [] }
    }
}
