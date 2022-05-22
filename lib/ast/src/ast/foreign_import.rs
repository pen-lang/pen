use super::calling_convention::CallingConvention;
use crate::types::Type;
use position::Position;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForeignImport {
    name: String,
    calling_convention: CallingConvention,
    type_: Type,
    position: Position,
}

impl ForeignImport {
    pub fn new(
        name: impl Into<String>,
        calling_convention: CallingConvention,
        type_: impl Into<Type>,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            calling_convention,
            type_: type_.into(),
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn calling_convention(&self) -> CallingConvention {
        self.calling_convention
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
