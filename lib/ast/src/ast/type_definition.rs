use super::{RecordDefinition, TypeAlias};
use position::Position;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeDefinition {
    RecordDefinition(RecordDefinition),
    TypeAlias(TypeAlias),
}

impl TypeDefinition {
    pub fn name(&self) -> &str {
        match self {
            Self::RecordDefinition(definition) => definition.name(),
            Self::TypeAlias(alias) => alias.name(),
        }
    }

    pub fn position(&self) -> &Position {
        match self {
            Self::RecordDefinition(definition) => definition.position(),
            Self::TypeAlias(alias) => alias.position(),
        }
    }
}

impl From<RecordDefinition> for TypeDefinition {
    fn from(definition: RecordDefinition) -> Self {
        Self::RecordDefinition(definition)
    }
}

impl From<TypeAlias> for TypeDefinition {
    fn from(alias: TypeAlias) -> Self {
        Self::TypeAlias(alias)
    }
}
