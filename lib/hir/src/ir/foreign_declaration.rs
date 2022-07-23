use super::calling_convention::CallingConvention;
use crate::types::Type;
use position::Position;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForeignDeclaration {
    name: String,
    foreign_name: String,
    calling_convention: CallingConvention,
    type_: Type,
    position: Position,
}

impl ForeignDeclaration {
    pub fn new(
        name: impl Into<String>,
        foreign_name: impl Into<String>,
        calling_convention: CallingConvention,
        type_: impl Into<Type>,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            foreign_name: foreign_name.into(),
            calling_convention,
            type_: type_.into(),
            position,
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

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
