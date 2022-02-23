#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallingConvention {
    C,
    Native,
    Trampoline,
}

impl Default for CallingConvention {
    fn default() -> Self {
        Self::Native
    }
}
