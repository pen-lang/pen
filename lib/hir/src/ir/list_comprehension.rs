use super::{ListComprehensionBranch, expression::Expression};
use crate::types::Type;
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct ListComprehension {
    type_: Type,
    element: Rc<Expression>,
    // Branches are in a Python-ish order of top to bottom.
    branches: Vec<ListComprehensionBranch>,
    position: Position,
}

impl ListComprehension {
    pub fn new(
        type_: impl Into<Type>,
        element: impl Into<Expression>,
        branches: Vec<ListComprehensionBranch>,
        position: Position,
    ) -> Self {
        Self {
            type_: type_.into(),
            element: element.into().into(),
            branches,
            position,
        }
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn element(&self) -> &Expression {
        &self.element
    }

    pub fn branches(&self) -> &[ListComprehensionBranch] {
        &self.branches
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
