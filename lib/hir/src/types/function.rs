use super::Type;
use position::Position;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Function(Arc<FunctionInner>);

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
struct FunctionInner {
    arguments: Arc<[Type]>,
    result: Type,
    position: Position,
}

impl Function {
    pub fn new(arguments: Vec<Type>, result: impl Into<Type>, position: Position) -> Self {
        Self(
            FunctionInner {
                arguments: arguments.into(),
                result: result.into(),
                position,
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

    pub fn position(&self) -> &Position {
        &self.0.position
    }

    pub fn set_position(self, position: Position) -> Self {
        Self(
            FunctionInner {
                arguments: self.0.arguments.clone(),
                result: self.0.result.clone(),
                position,
            }
            .into(),
        )
    }
}
