use super::CallingConvention;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForeignDefinition {
    name: String,
    foreign_name: String,
    calling_convention: CallingConvention,
}

impl ForeignDefinition {
    pub fn new(
        name: impl Into<String>,
        foreign_name: impl Into<String>,
        calling_convention: CallingConvention,
    ) -> Self {
        Self {
            name: name.into(),
            foreign_name: foreign_name.into(),
            calling_convention,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn foreign_name(&self) -> &str {
        &self.foreign_name
    }

    pub fn calling_convention(&self) -> CallingConvention {
        self.calling_convention
    }
}
