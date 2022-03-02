use super::Type;
use core::fmt;
use position::Position;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    sync::Arc,
};

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct List {
    element: Arc<Type>,
    position: Position,
}

impl List {
    pub fn new(element: impl Into<Type>, position: Position) -> Self {
        Self {
            element: Arc::new(element.into()),
            position,
        }
    }

    pub fn element(&self) -> &Type {
        &self.element
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}

impl Display for List {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "[{}]", &self.element)
    }
}
