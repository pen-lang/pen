use crate::ir::*;
use position::{Position, test::PositionFake};

pub trait ModuleFake {
    fn empty() -> Self;

    #[must_use]
    fn set_type_definitions(&self, type_definitions: Vec<TypeDefinition>) -> Self;

    #[must_use]
    fn set_type_aliases(&self, type_aliases: Vec<TypeAlias>) -> Self;

    #[must_use]
    fn set_foreign_declarations(&self, declarations: Vec<ForeignDeclaration>) -> Self;

    #[must_use]
    fn set_function_declarations(&self, declarations: Vec<FunctionDeclaration>) -> Self;

    #[must_use]
    fn set_function_definitions(&self, definitions: Vec<FunctionDefinition>) -> Self;
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
            self.function_declarations().to_vec(),
            self.function_definitions().to_vec(),
            self.position().clone(),
        )
    }

    fn set_type_aliases(&self, type_aliases: Vec<TypeAlias>) -> Self {
        Self::new(
            self.type_definitions().to_vec(),
            type_aliases,
            self.foreign_declarations().to_vec(),
            self.function_declarations().to_vec(),
            self.function_definitions().to_vec(),
            self.position().clone(),
        )
    }

    fn set_foreign_declarations(&self, declarations: Vec<ForeignDeclaration>) -> Self {
        Self::new(
            self.type_definitions().to_vec(),
            self.type_aliases().to_vec(),
            declarations,
            self.function_declarations().to_vec(),
            self.function_definitions().to_vec(),
            self.position().clone(),
        )
    }

    fn set_function_declarations(&self, declarations: Vec<FunctionDeclaration>) -> Self {
        Self::new(
            self.type_definitions().to_vec(),
            self.type_aliases().to_vec(),
            self.foreign_declarations().to_vec(),
            declarations,
            self.function_definitions().to_vec(),
            self.position().clone(),
        )
    }

    fn set_function_definitions(&self, definitions: Vec<FunctionDefinition>) -> Self {
        Self::new(
            self.type_definitions().to_vec(),
            self.type_aliases().to_vec(),
            self.foreign_declarations().to_vec(),
            self.function_declarations().to_vec(),
            definitions,
            self.position().clone(),
        )
    }
}
