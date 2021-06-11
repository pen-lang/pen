use crate::types::Type;

#[derive(Clone, Debug, PartialEq)]
pub struct Argument {
    name: String,
    type_: Type,
}

impl Argument {
    pub fn new(name: impl Into<String>, type_: impl Into<Type>) -> Self {
        Self {
            name: name.into(),
            type_: type_.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }
}
