use super::calling_convention::CallingConvention;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForeignDefinitionConfiguration {
    calling_convention: CallingConvention,
}

impl ForeignDefinitionConfiguration {
    pub fn new(calling_convention: CallingConvention) -> Self {
        Self { calling_convention }
    }

    pub fn calling_convention(&self) -> CallingConvention {
        self.calling_convention
    }
}
