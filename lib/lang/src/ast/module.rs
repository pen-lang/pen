use super::{definition::Definition, type_definition::TypeDefinition, ForeignImport, Import};
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    imports: Vec<Import>,
    foreign_imports: Vec<ForeignImport>,
    type_definitions: Vec<TypeDefinition>,
    definitions: Vec<Definition>,
    position: Position,
}

impl Module {
    pub fn new(
        imports: Vec<Import>,
        foreign_imports: Vec<ForeignImport>,
        type_definitions: Vec<TypeDefinition>,
        definitions: Vec<Definition>,
        position: Position,
    ) -> Self {
        Self {
            imports,
            foreign_imports,
            type_definitions,
            definitions,
            position,
        }
    }

    pub fn imports(&self) -> &[Import] {
        &self.imports
    }

    pub fn foreign_imports(&self) -> &[ForeignImport] {
        &self.foreign_imports
    }

    pub fn type_definitions(&self) -> &[TypeDefinition] {
        &self.type_definitions
    }

    pub fn definitions(&self) -> &[Definition] {
        &self.definitions
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
