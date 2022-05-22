use super::calling_convention::CallingConvention;
use crate::types;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForeignDeclaration {
    name: String,
    foreign_name: String,
    type_: types::Function,
    calling_convention: CallingConvention,
}

impl ForeignDeclaration {
    pub fn new(
        name: impl Into<String>,
        foreign_name: impl Into<String>,
        type_: types::Function,
        calling_convention: CallingConvention,
    ) -> Self {
        Self {
            name: name.into(),
            foreign_name: foreign_name.into(),
            type_,
            calling_convention,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn foreign_name(&self) -> &str {
        &self.foreign_name
    }

    pub fn type_(&self) -> &types::Function {
        &self.type_
    }

    pub fn calling_convention(&self) -> CallingConvention {
        self.calling_convention
    }
}
