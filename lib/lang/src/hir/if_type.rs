use super::{expression::Expression, if_type_branch::IfTypeBranch, Block};
use crate::{position::Position, types::Type};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct IfType {
    name: String,
    argument: Arc<Expression>,
    argument_type: Option<Type>,
    branches: Vec<IfTypeBranch>,
    else_: Option<Block>,
    result_type: Option<Type>,
    position: Position,
}

impl IfType {
    pub fn new(
        name: impl Into<String>,
        argument: impl Into<Expression>,
        argument_type: Option<Type>,
        branches: Vec<IfTypeBranch>,
        else_: Option<Block>,
        result_type: Option<Type>,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            argument: argument.into().into(),
            argument_type,
            branches,
            else_,
            result_type,
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn argument(&self) -> &Expression {
        &self.argument
    }

    pub fn argument_type(&self) -> Option<&Type> {
        self.argument_type.as_ref()
    }

    pub fn branches(&self) -> &[IfTypeBranch] {
        &self.branches
    }

    pub fn else_(&self) -> Option<&Block> {
        self.else_.as_ref()
    }

    pub fn result_type(&self) -> Option<&Type> {
        self.result_type.as_ref()
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
