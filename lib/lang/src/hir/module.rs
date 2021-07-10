use super::{Declaration, Definition, ForeignFunctionDeclaration, TypeAlias, TypeDefinition};
use crate::position::Position;

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    type_definitions: Vec<TypeDefinition>,
    type_aliases: Vec<TypeAlias>,
    foreign_function_declarations: Vec<ForeignFunctionDeclaration>,
    declarations: Vec<Declaration>,
    definitions: Vec<Definition>,
    position: Position,
}

impl Module {
    pub fn new(
        type_definitions: Vec<TypeDefinition>,
        type_aliases: Vec<TypeAlias>,
        foreign_function_declarations: Vec<ForeignFunctionDeclaration>,
        declarations: Vec<Declaration>,
        definitions: Vec<Definition>,
        position: Position,
    ) -> Self {
        Self {
            type_definitions,
            type_aliases,
            foreign_function_declarations,
            declarations,
            definitions,
            position,
        }
    }

    #[cfg(test)]
    pub fn empty() -> Self {
        Self::new(vec![], vec![], vec![], vec![], vec![], Position::dummy())
    }

    #[cfg(test)]
    pub fn set_type_definitions(&self, type_definitions: Vec<TypeDefinition>) -> Self {
        Self::new(
            type_definitions,
            self.type_aliases.clone(),
            self.foreign_function_declarations.clone(),
            self.declarations.clone(),
            self.definitions.clone(),
            self.position.clone(),
        )
    }

    #[cfg(test)]
    pub fn set_type_aliases(&self, type_aliases: Vec<TypeAlias>) -> Self {
        Self::new(
            self.type_definitions.clone(),
            type_aliases,
            self.foreign_function_declarations.clone(),
            self.declarations.clone(),
            self.definitions.clone(),
            self.position.clone(),
        )
    }

    #[cfg(test)]
    pub fn set_foreign_function_declarations(&self, declarations: Vec<ForeignFunctionDeclaration>) -> Self {
        Self::new(
            self.type_definitions.clone(),
            self.type_aliases.clone(),
            declarations,
            self.declarations.clone(),
            self.definitions.clone(),
            self.position.clone(),
        )
    }

    #[cfg(test)]
    pub fn set_declarations(&self, declarations: Vec<Declaration>) -> Self {
        Self::new(
            self.type_definitions.clone(),
            self.type_aliases.clone(),
            self.foreign_function_declarations.clone(),
            declarations,
            self.definitions.clone(),
            self.position.clone(),
        )
    }

    #[cfg(test)]
    pub fn set_definitions(&self, definitions: Vec<Definition>) -> Self {
        Self::new(
            self.type_definitions.clone(),
            self.type_aliases.clone(),
            self.foreign_function_declarations.clone(),
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

    pub fn foreign_function_declarations(&self) -> &[ForeignFunctionDeclaration] {
        &self.foreign_function_declarations
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
