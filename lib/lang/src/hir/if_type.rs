use super::{expression::Expression, if_type_branch::IfTypeBranch};
use crate::position::Position;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct IfType {
    name: String,
    argument: Arc<Expression>,
    branches: Vec<IfTypeBranch>,
    else_: Arc<Option<Expression>>,
    position: Position,
}

impl IfType {
    pub fn new(
        name: impl Into<String>,
        argument: impl Into<Expression>,
        branches: Vec<IfTypeBranch>,
        else_: Option<Expression>,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            argument: argument.into().into(),
            branches,
            else_: else_.into(),
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

    pub fn else_(&self) -> Option<&Expression> {
        self.else_.as_ref().as_ref()
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
