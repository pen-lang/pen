use super::{definition::Definition, type_definition::TypeDefinition, Declaration, TypeAlias};

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    external_type_definitions: Vec<TypeDefinition>,
    external_type_aliases: Vec<TypeAlias>,
    type_definitions: Vec<TypeDefinition>,
    type_aliases: Vec<TypeAlias>,
    declarations: Vec<Declaration>,
    definitions: Vec<Definition>,
}

impl Module {
    pub fn new(
        external_type_definitions: Vec<TypeDefinition>,
        external_type_aliases: Vec<TypeAlias>,
        type_definitions: Vec<TypeDefinition>,
        type_aliases: Vec<TypeAlias>,
        declarations: Vec<Declaration>,
        definitions: Vec<Definition>,
    ) -> Self {
        Self {
            external_type_definitions,
            external_type_aliases,
            type_definitions,
            type_aliases,
            declarations,
            definitions,
        }
    }

    pub fn external_type_definitions(&self) -> &[TypeDefinition] {
        &self.external_type_definitions
    }

    pub fn external_type_aliases(&self) -> &[TypeAlias] {
        &self.external_type_aliases
    }

    pub fn type_definitions(&self) -> &[TypeDefinition] {
        &self.type_definitions
    }

    pub fn type_aliases(&self) -> &[TypeAlias] {
        &self.type_aliases
    }

    pub fn declarations(&self) -> &[Declaration] {
        &self.declarations
    }

    pub fn definitions(&self) -> &[Definition] {
        &self.definitions
    }
}
