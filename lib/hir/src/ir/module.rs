use super::{
    definition::Definition, type_definition::TypeDefinition, Declaration, ForeignDeclaration,
    TypeAlias,
};
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    type_definitions: Vec<TypeDefinition>,
    type_aliases: Vec<TypeAlias>,
    foreign_declarations: Vec<ForeignDeclaration>,
    declarations: Vec<Declaration>,
    definitions: Vec<Definition>,
    position: Position,
}

impl Module {
    pub fn new(
        type_definitions: Vec<TypeDefinition>,
        type_aliases: Vec<TypeAlias>,
        foreign_declarations: Vec<ForeignDeclaration>,
        declarations: Vec<Declaration>,
        definitions: Vec<Definition>,
        position: Position,
    ) -> Self {
        Self {
            type_definitions,
            type_aliases,
            foreign_declarations,
            declarations,
            definitions,
            position,
        }
    }

    pub fn type_definitions(&self) -> &[TypeDefinition] {
        &self.type_definitions
    }

    pub fn type_aliases(&self) -> &[TypeAlias] {
        &self.type_aliases
    }

    pub fn foreign_declarations(&self) -> &[ForeignDeclaration] {
        &self.foreign_declarations
    }

    pub fn declarations(&self) -> &[Declaration] {
        &self.declarations
    }

    pub fn definitions(&self) -> &[Definition] {
        &self.definitions
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
