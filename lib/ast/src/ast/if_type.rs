use super::{Block, expression::Expression, if_type_branch::IfTypeBranch};
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct IfType {
    name: String,
    argument: Rc<Expression>,
    branches: Vec<IfTypeBranch>,
    else_: Option<Block>,
    position: Position,
}

impl IfType {
    pub fn new(
        name: impl Into<String>,
        argument: impl Into<Expression>,
        branches: Vec<IfTypeBranch>,
        else_: Option<Block>,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            argument: argument.into().into(),
            branches,
            else_,
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn argument(&self) -> &Expression {
        &self.argument
    }

    pub fn branches(&self) -> &[IfTypeBranch] {
        &self.branches
    }

    pub fn else_(&self) -> Option<&Block> {
        self.else_.as_ref()
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
