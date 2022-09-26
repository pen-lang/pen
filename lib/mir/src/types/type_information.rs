use super::Type;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeInformation {
    types: Vec<Type>,
}

impl TypeInformation {
    pub fn new(types: Vec<Type>) -> Self {
        Self { types }
    }

    pub fn types(&self) -> &[Type] {
        &self.types
    }
}
