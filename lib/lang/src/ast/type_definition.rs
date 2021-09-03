use super::{RecordDefinition, TypeAlias};

#[derive(Clone, Debug, PartialEq)]
pub enum TypeDefinition {
    RecordDefinition(RecordDefinition),
    TypeAlias(TypeAlias),
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
