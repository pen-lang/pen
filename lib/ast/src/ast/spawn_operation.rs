use super::lambda::Lambda;
use position::Position;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct SpawnOperation {
    function: Arc<Lambda>,
    position: Position,
}

impl SpawnOperation {
    pub fn new(function: Lambda, position: Position) -> Self {
        Self {
            function: function.into(),
            position,
        }
    }

    pub fn function(&self) -> &Lambda {
        &self.function
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
