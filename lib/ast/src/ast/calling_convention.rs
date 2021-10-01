#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallingConvention {
    Native,
    C,
}

impl Default for CallingConvention {
    fn default() -> Self {
        Self::Native
    }
}
