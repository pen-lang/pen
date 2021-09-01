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

    // TODO Move to a hir_test crate.
    pub fn empty() -> Self {
        Self::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            crate::test::position(),
        )
    }

    // TODO Move to a hir_test crate.
    pub fn set_type_definitions(&self, type_definitions: Vec<TypeDefinition>) -> Self {
        Self::new(
            type_definitions,
            self.type_aliases.clone(),
            self.foreign_declarations.clone(),
            self.declarations.clone(),
            self.definitions.clone(),
            self.position.clone(),
        )
    }

    // TODO Move to a hir_test crate.
    pub fn set_type_aliases(&self, type_aliases: Vec<TypeAlias>) -> Self {
        Self::new(
            self.type_definitions.clone(),
            type_aliases,
            self.foreign_declarations.clone(),
            self.declarations.clone(),
            self.definitions.clone(),
            self.position.clone(),
        )
    }

    // TODO Move to a hir_test crate.
    pub fn set_foreign_declarations(&self, declarations: Vec<ForeignDeclaration>) -> Self {
        Self::new(
            self.type_definitions.clone(),
            self.type_aliases.clone(),
            declarations,
            self.declarations.clone(),
            self.definitions.clone(),
            self.position.clone(),
        )
    }

    // TODO Move to a hir_test crate.
    pub fn set_declarations(&self, declarations: Vec<Declaration>) -> Self {
        Self::new(
            self.type_definitions.clone(),
            self.type_aliases.clone(),
            self.foreign_declarations.clone(),
            declarations,
            self.definitions.clone(),
            self.position.clone(),
        )
    }

    // TODO Move to a hir_test crate.
    pub fn set_definitions(&self, definitions: Vec<Definition>) -> Self {
        Self::new(
            self.type_definitions.clone(),
            self.type_aliases.clone(),
            self.foreign_declarations.clone(),
            self.declarations.clone(),
            definitions,
            self.position.clone(),
        )
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
