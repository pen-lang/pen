use super::Type;
use core::fmt;
use position::Position;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    sync::Arc,
};

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Function {
    arguments: Vec<Type>,
    result: Arc<Type>,
    position: Position,
}

impl Function {
    pub fn new(arguments: Vec<Type>, result: impl Into<Type>, position: Position) -> Self {
        Self {
            arguments,
            result: Arc::new(result.into()),
            position,
        }
    }

    pub fn arguments(&self) -> &[Type] {
        &self.arguments
    }

    pub fn result(&self) -> &Type {
        &self.result
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}

impl Display for Function {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(
            formatter,
            "\\({}) {}",
            &self
                .arguments
                .iter()
                .map(|type_| format!("{}", type_))
                .collect::<Vec<_>>()
                .join(", "),
            &self.result
        )
    }
}
