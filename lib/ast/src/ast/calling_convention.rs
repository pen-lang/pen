#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum CallingConvention {
    #[default]
    Native,
    C,
}
