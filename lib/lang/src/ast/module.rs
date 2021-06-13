use super::{definition::Definition, type_definition::TypeDefinition, Import, TypeAlias};

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    imports: Vec<Import>,
    type_definitions: Vec<TypeDefinition>,
    type_aliases: Vec<TypeAlias>,
    definitions: Vec<Definition>,
}

impl Module {
    pub fn new(
        imports: Vec<Import>,
        type_definitions: Vec<TypeDefinition>,
        type_aliases: Vec<TypeAlias>,
        definitions: Vec<Definition>,
    ) -> Self {
        Self {
            imports,
            type_definitions,
            type_aliases,
            definitions,
        }
    }

    pub fn imports(&self) -> &[Import] {
        &self.imports
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
