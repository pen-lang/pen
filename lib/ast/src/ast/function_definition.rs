use super::{foreign_export::ForeignExport, lambda::Lambda};
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDefinition {
    name: String,
    lambda: Lambda,
    foreign_export: Option<ForeignExport>,
    position: Position,
}

impl FunctionDefinition {
    pub fn new(
        name: impl Into<String>,
        lambda: Lambda,
        foreign_export: Option<ForeignExport>,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            lambda,
            foreign_export,
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn lambda(&self) -> &Lambda {
        &self.lambda
    }

    pub fn foreign_export(&self) -> Option<&ForeignExport> {
        self.foreign_export.as_ref()
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
