use crate::types;

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDeclaration {
    name: String,
    type_: types::Function,
}

impl FunctionDeclaration {
    pub fn new(name: impl Into<String>, type_: types::Function) -> Self {
        Self {
            name: name.into(),
            type_,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_(&self) -> &types::Function {
        &self.type_
    }
}
