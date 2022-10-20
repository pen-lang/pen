use super::Type;
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Map {
    key: Rc<Type>,
    value: Rc<Type>,
    position: Position,
}

impl Map {
    pub fn new(key: impl Into<Type>, value: impl Into<Type>, position: Position) -> Self {
        Self {
            key: Rc::new(key.into()),
            value: Rc::new(value.into()),
            position,
        }
    }

    pub fn key(&self) -> &Type {
        &self.key
    }

    pub fn value(&self) -> &Type {
        &self.value
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
