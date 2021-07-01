use super::{
    definition::Definition, type_definition::TypeDefinition, ForeignImport, Import, TypeAlias,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    imports: Vec<Import>,
    foreign_imports: Vec<ForeignImport>,
    type_definitions: Vec<TypeDefinition>,
    type_aliases: Vec<TypeAlias>,
    definitions: Vec<Definition>,
}

impl Module {
    pub fn new(
        imports: Vec<Import>,
        foreign_imports: Vec<ForeignImport>,
        type_definitions: Vec<TypeDefinition>,
        type_aliases: Vec<TypeAlias>,
        definitions: Vec<Definition>,
    ) -> Self {
        Self {
            imports,
            foreign_imports,
            type_definitions,
            type_aliases,
            definitions,
        }
    }

    pub fn imports(&self) -> &[Import] {
        &self.imports
    }

    pub fn foreign_imports(&self) -> &[Import] {
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
