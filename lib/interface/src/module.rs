use super::{FunctionDeclaration, TypeAlias, TypeDefinition};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Module {
    type_definitions: Vec<TypeDefinition>,
    type_aliases: Vec<TypeAlias>,
    function_declarations: Vec<FunctionDeclaration>,
}

impl Module {
    pub fn new(
        type_definitions: Vec<TypeDefinition>,
        type_aliases: Vec<TypeAlias>,
        declarations: Vec<FunctionDeclaration>,
    ) -> Self {
        Self {
            type_definitions,
            type_aliases,
            function_declarations: declarations,
        }
    }

    pub fn type_definitions(&self) -> &[TypeDefinition] {
        &self.type_definitions
    }

    pub fn type_aliases(&self) -> &[TypeAlias] {
        &self.type_aliases
    }

    pub fn function_declarations(&self) -> &[FunctionDeclaration] {
        &self.function_declarations
    }
}
