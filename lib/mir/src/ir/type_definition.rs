use crate::types;

#[derive(Clone, Debug, PartialEq)]
pub struct TypeDefinition {
    name: String,
    type_: types::RecordBody,
}

impl TypeDefinition {
    pub fn new(name: impl Into<String>, type_: impl Into<types::RecordBody>) -> Self {
        Self {
            name: name.into(),
            type_: type_.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_(&self) -> &types::RecordBody {
        &self.type_
    }
}
