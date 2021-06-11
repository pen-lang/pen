use super::{Declaration, TypeAlias, TypeDefinition};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct ModuleInterface {
    type_definitions: Vec<TypeDefinition>,
    type_aliases: Vec<TypeAlias>,
    declarations: Vec<Declaration>,
}

impl ModuleInterface {
    pub fn new(
        type_definitions: Vec<TypeDefinition>,
        type_aliases: Vec<TypeAlias>,
        declarations: Vec<Declaration>,
    ) -> Self {
        Self {
            type_definitions,
            type_aliases,
            declarations,
        }
    }

    pub fn type_definitions(&self) -> &[TypeDefinition] {
        &self.type_definitions
    }

    pub fn type_aliases(&self) -> &[TypeAlias] {
        &self.type_aliases
    }

    pub fn declarations(&self) -> &[Declaration] {
        &self.declarations
    }
}
