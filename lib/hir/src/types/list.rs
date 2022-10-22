use super::Type;
use position::Position;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct List(Rc<ListInner>);

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
struct ListInner {
    element: Type,
    position: Position,
}

impl List {
    pub fn new(element: impl Into<Type>, position: Position) -> Self {
        Self(
            ListInner {
                element: element.into(),
                position,
            }
            .into(),
        )
    }

    pub fn element(&self) -> &Type {
        &self.0.element
    }

    pub fn position(&self) -> &Position {
        &self.0.position
    }

    pub fn set_position(&self, position: Position) -> Self {
        Self(
            ListInner {
                element: self.0.element.clone(),
                position,
            }
            .into(),
        )
    }
}
