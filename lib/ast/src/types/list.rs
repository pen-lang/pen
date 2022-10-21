use super::Type;
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct List {
    element: Rc<Type>,
    position: Position,
}

impl List {
    pub fn new(element: impl Into<Type>, position: Position) -> Self {
        Self {
            element: Rc::new(element.into()),
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
