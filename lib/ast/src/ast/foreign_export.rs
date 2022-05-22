use crate::CallingConvention;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForeignExport {
    calling_convention: CallingConvention,
}

impl ForeignExport {
    pub fn new(calling_convention: CallingConvention) -> Self {
        Self { calling_convention }
    }

    pub fn calling_convention(&self) -> CallingConvention {
        self.calling_convention
    }
}
