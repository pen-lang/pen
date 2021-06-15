use super::{Block, IfBranch};
use crate::position::Position;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct If {
    branches: Vec<IfBranch>,
    else_: Arc<Block>,
    position: Position,
}

impl If {
    pub fn new(branches: Vec<IfBranch>, else_: Block, position: Position) -> Self {
        Self {
            branches,
            else_: else_.into(),
            position,
        }
    }

    pub fn branches(&self) -> &[IfBranch] {
        &self.branches
    }

    pub fn else_(&self) -> &Block {
        &self.else_
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
