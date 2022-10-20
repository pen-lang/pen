use super::type_::Type;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Function(Arc<FunctionInner>);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct FunctionInner {
    arguments: Vec<Type>,
    result: Type,
}

impl Function {
    pub fn new(arguments: Vec<Type>, result: impl Into<Type>) -> Self {
        Self(
            FunctionInner {
                arguments,
                result: result.into(),
            }
            .into(),
        )
    }

    pub fn arguments(&self) -> &[Type] {
        &self.0.arguments
    }

    pub fn result(&self) -> &Type {
        &self.0.result
    }
}
