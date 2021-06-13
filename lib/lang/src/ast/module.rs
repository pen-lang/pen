use super::{definition::Definition, type_definition::TypeDefinition, TypeAlias};

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    type_definitions: Vec<TypeDefinition>,
    type_aliases: Vec<TypeAlias>,
    definitions: Vec<Definition>,
}

impl Module {
    pub fn new(
        type_definitions: Vec<TypeDefinition>,
        type_aliases: Vec<TypeAlias>,
        definitions: Vec<Definition>,
    ) -> Self {
        Self {
            type_definitions,
            type_aliases,
            definitions,
        }
    }

    pub fn type_definitions(&self) -> &[TypeDefinition] {
        &self.type_definitions
    }

    pub fn type_aliases(&self) -> &[TypeAlias] {
        &self.type_aliases
    }

    pub fn definitions(&self) -> &[Definition] {
        &self.definitions
    }
}
