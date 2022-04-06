use super::{FunctionDeclaration, RecordDefinition, TypeAlias};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Module {
    type_definitions: Vec<RecordDefinition>,
    type_aliases: Vec<TypeAlias>,
    function_declarations: Vec<FunctionDeclaration>,
}

impl Module {
    pub fn new(
        type_definitions: Vec<RecordDefinition>,
        type_aliases: Vec<TypeAlias>,
        declarations: Vec<FunctionDeclaration>,
    ) -> Self {
        Self {
            type_definitions,
            type_aliases,
            function_declarations: declarations,
        }
    }

    pub fn type_definitions(&self) -> &[RecordDefinition] {
        &self.type_definitions
    }

    pub fn type_aliases(&self) -> &[TypeAlias] {
        &self.type_aliases
    }

    pub fn function_declarations(&self) -> &[FunctionDeclaration] {
        &self.function_declarations
    }
}
