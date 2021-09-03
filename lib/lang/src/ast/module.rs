use super::{
    definition::Definition, record_definition::RecordDefinition, ForeignImport, Import, TypeAlias,
};
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    imports: Vec<Import>,
    foreign_imports: Vec<ForeignImport>,
    record_definitions: Vec<RecordDefinition>,
    type_aliases: Vec<TypeAlias>,
    definitions: Vec<Definition>,
    position: Position,
}

impl Module {
    pub fn new(
        imports: Vec<Import>,
        foreign_imports: Vec<ForeignImport>,
        record_definitions: Vec<RecordDefinition>,
        type_aliases: Vec<TypeAlias>,
        definitions: Vec<Definition>,
        position: Position,
    ) -> Self {
        Self {
            imports,
            foreign_imports,
            record_definitions,
            type_aliases,
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

    pub fn record_definitions(&self) -> &[RecordDefinition] {
        &self.record_definitions
    }

    pub fn type_aliases(&self) -> &[TypeAlias] {
        &self.type_aliases
    }

    pub fn definitions(&self) -> &[Definition] {
        &self.definitions
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
