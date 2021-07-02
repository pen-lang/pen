use super::{
    definition::Definition, type_definition::TypeDefinition, Declaration, ForeignDeclaration,
    TypeAlias,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    type_definitions: Vec<TypeDefinition>,
    type_aliases: Vec<TypeAlias>,
    foreign_declarations: Vec<ForeignDeclaration>,
    declarations: Vec<Declaration>,
    definitions: Vec<Definition>,
}

impl Module {
    pub fn new(
        type_definitions: Vec<TypeDefinition>,
        type_aliases: Vec<TypeAlias>,
        foreign_declarations: Vec<ForeignDeclaration>,
        declarations: Vec<Declaration>,
        definitions: Vec<Definition>,
    ) -> Self {
        Self {
            type_definitions,
            type_aliases,
            foreign_declarations,
            declarations,
            definitions,
        }
    }

    #[cfg(test)]
    pub fn empty() -> Self {
        Self::new(vec![], vec![], vec![], vec![], vec![])
    }

    #[cfg(test)]
    pub fn set_type_definitions(&self, type_definitions: Vec<TypeDefinition>) -> Self {
        Self::new(
            type_definitions,
            self.type_aliases.clone(),
            self.foreign_declarations.clone(),
            self.declarations.clone(),
            self.definitions.clone(),
        )
    }

    #[cfg(test)]
    pub fn set_type_aliases(&self, type_aliases: Vec<TypeAlias>) -> Self {
        Self::new(
            self.type_definitions.clone(),
            type_aliases,
            self.foreign_declarations.clone(),
            self.declarations.clone(),
            self.definitions.clone(),
        )
    }

    #[cfg(test)]
    pub fn set_declarations(&self, declarations: Vec<Declaration>) -> Self {
        Self::new(
            self.type_definitions.clone(),
            self.type_aliases.clone(),
            self.foreign_declarations.clone(),
            declarations,
            self.definitions.clone(),
        )
    }

    #[cfg(test)]
    pub fn set_definitions(&self, definitions: Vec<Definition>) -> Self {
        Self::new(
            self.type_definitions.clone(),
            self.type_aliases.clone(),
            self.foreign_declarations.clone(),
            self.declarations.clone(),
            definitions,
        )
    }

    #[cfg(test)]
    pub fn from_definitions(definitions: Vec<Definition>) -> Self {
        Self::new(vec![], vec![], vec![], vec![], definitions)
    }

    #[cfg(test)]
    pub fn from_type_definitions_and_definitions(
        type_definitions: Vec<TypeDefinition>,
        definitions: Vec<Definition>,
    ) -> Self {
        Self::new(type_definitions, vec![], vec![], vec![], definitions)
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
}
