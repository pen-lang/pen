use super::Type;
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Function {
    arguments: Vec<Type>,
    result: Rc<Type>,
    position: Position,
}

impl Function {
    pub fn new(arguments: Vec<Type>, result: impl Into<Type>, position: Position) -> Self {
        Self {
            arguments,
            result: Rc::new(result.into()),
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
