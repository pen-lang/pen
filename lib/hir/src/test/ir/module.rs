use crate::ir::*;
use position::{test::PositionFake, Position};

pub trait ModuleFake {
    fn empty() -> Self;
    fn set_type_definitions(&self, type_definitions: Vec<TypeDefinition>) -> Self;
    fn set_type_aliases(&self, type_aliases: Vec<TypeAlias>) -> Self;
    fn set_foreign_declarations(&self, declarations: Vec<ForeignDeclaration>) -> Self;
    fn set_declarations(&self, declarations: Vec<Declaration>) -> Self;
    fn set_definitions(&self, definitions: Vec<Definition>) -> Self;
}

impl ModuleFake for Module {
    fn empty() -> Self {
        Self::new(vec![], vec![], vec![], vec![], vec![], Position::fake())
    }

    fn set_type_definitions(&self, type_definitions: Vec<TypeDefinition>) -> Self {
        Self::new(
            type_definitions,
            self.type_aliases().to_vec(),
            self.foreign_declarations().to_vec(),
            self.declarations().to_vec(),
            self.definitions().to_vec(),
            self.position().clone(),
        )
    }

    fn set_type_aliases(&self, type_aliases: Vec<TypeAlias>) -> Self {
        Self::new(
            self.type_definitions().to_vec(),
            type_aliases,
            self.foreign_declarations().to_vec(),
            self.declarations().to_vec(),
            self.definitions().to_vec(),
            self.position().clone(),
        )
    }

    fn set_foreign_declarations(&self, declarations: Vec<ForeignDeclaration>) -> Self {
        Self::new(
            self.type_definitions().to_vec(),
            self.type_aliases().to_vec(),
            declarations,
            self.declarations().to_vec(),
            self.definitions().to_vec(),
            self.position().clone(),
        )
    }

    fn set_declarations(&self, declarations: Vec<Declaration>) -> Self {
        Self::new(
            self.type_definitions().to_vec(),
            self.type_aliases().to_vec(),
            self.foreign_declarations().to_vec(),
            declarations,
            self.definitions().to_vec(),
            self.position().clone(),
        )
    }

    fn set_definitions(&self, definitions: Vec<Definition>) -> Self {
        Self::new(
            self.type_definitions().to_vec(),
            self.type_aliases().to_vec(),
            self.foreign_declarations().to_vec(),
            self.declarations().to_vec(),
            definitions,
            self.position().clone(),
        )
    }
}
