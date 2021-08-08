use crate::type_information;

#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq)]
pub struct None {
    _private: [u8; 0],
}

impl None {
    pub fn new() -> Self {
        Self { _private: [] }
    }
}

type_information!(byte_string, crate::none::None);
