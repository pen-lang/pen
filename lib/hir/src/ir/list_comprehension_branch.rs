use super::expression::Expression;
use crate::types::Type;
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct ListComprehensionBranch {
    type_: Option<Type>,
    primary_name: String,
    secondary_name: Option<String>,
    iteratee: Rc<Expression>,
    position: Position,
}

impl ListComprehensionBranch {
    pub fn new(
        type_: Option<Type>,
        primary_name: impl Into<String>,
        secondary_name: Option<String>,
        iteratee: impl Into<Expression>,
        position: Position,
    ) -> Self {
        Self {
            type_,
            primary_name: primary_name.into(),
            secondary_name,
            iteratee: iteratee.into().into(),
            position,
        }
    }

    pub fn type_(&self) -> Option<&Type> {
        self.type_.as_ref()
    }

    pub fn primary_name(&self) -> &str {
        &self.primary_name
    }

    pub fn secondary_name(&self) -> Option<&str> {
        self.secondary_name.as_deref()
    }

    pub fn iteratee(&self) -> &Expression {
        &self.iteratee
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
