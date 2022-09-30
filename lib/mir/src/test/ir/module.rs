use super::GlobalFunctionDefinitionFake;
use crate::ir::*;

pub trait ModuleFake {
    #[must_use]
    fn empty() -> Self;

    #[must_use]
    fn set_type_definitions(&self, definitions: Vec<TypeDefinition>) -> Self;

    #[must_use]
    fn set_foreign_declarations(&self, declarations: Vec<ForeignDeclaration>) -> Self;

    #[must_use]
    fn set_foreign_definitions(&self, definitions: Vec<ForeignDefinition>) -> Self;

    #[must_use]
    fn set_function_declarations(&self, declarations: Vec<FunctionDeclaration>) -> Self;

    #[must_use]
    fn set_function_definitions(&self, definitions: Vec<FunctionDefinition>) -> Self;

    #[must_use]
    fn set_global_function_definitions(&self, definitions: Vec<GlobalFunctionDefinition>) -> Self;

    #[must_use]
    fn set_type_information(&self, information: TypeInformation) -> Self;
}

impl ModuleFake for Module {
    fn empty() -> Self {
        Self::new(vec![], vec![], vec![], vec![], vec![], Default::default())
    }

    fn set_type_definitions(&self, definitions: Vec<TypeDefinition>) -> Self {
        Self::new(
            definitions,
            self.foreign_declarations().to_vec(),
            self.foreign_definitions().to_vec(),
            self.function_declarations().to_vec(),
            self.function_definitions().to_vec(),
            self.type_information().clone(),
        )
    }

    fn set_foreign_declarations(&self, declarations: Vec<ForeignDeclaration>) -> Self {
        Self::new(
            self.type_definitions().to_vec(),
            declarations,
            self.foreign_definitions().to_vec(),
            self.function_declarations().to_vec(),
            self.function_definitions().to_vec(),
            self.type_information().clone(),
        )
    }

    fn set_foreign_definitions(&self, definitions: Vec<ForeignDefinition>) -> Self {
        Self::new(
            self.type_definitions().to_vec(),
            self.foreign_declarations().to_vec(),
            definitions,
            self.function_declarations().to_vec(),
            self.function_definitions().to_vec(),
            self.type_information().clone(),
        )
    }

    fn set_function_declarations(&self, declarations: Vec<FunctionDeclaration>) -> Self {
        Self::new(
            self.type_definitions().to_vec(),
            self.foreign_declarations().to_vec(),
            self.foreign_definitions().to_vec(),
            declarations,
            self.function_definitions().to_vec(),
            self.type_information().clone(),
        )
    }

    fn set_function_definitions(&self, definitions: Vec<FunctionDefinition>) -> Self {
        Self::new(
            self.type_definitions().to_vec(),
            self.foreign_declarations().to_vec(),
            self.foreign_definitions().to_vec(),
            self.function_declarations().to_vec(),
            definitions
                .into_iter()
                .map(GlobalFunctionDefinition::fake)
                .collect(),
            self.type_information().clone(),
        )
    }

    fn set_global_function_definitions(&self, definitions: Vec<GlobalFunctionDefinition>) -> Self {
        Self::new(
            self.type_definitions().to_vec(),
            self.foreign_declarations().to_vec(),
            self.foreign_definitions().to_vec(),
            self.function_declarations().to_vec(),
            definitions,
            self.type_information().clone(),
        )
    }

    fn set_type_information(&self, type_information: TypeInformation) -> Self {
        Self::new(
            self.type_definitions().to_vec(),
            self.foreign_declarations().to_vec(),
            self.foreign_definitions().to_vec(),
            self.function_declarations().to_vec(),
            self.function_definitions().to_vec(),
            type_information,
        )
    }
}
