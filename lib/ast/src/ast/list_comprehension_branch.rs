use super::expression::Expression;
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct ListComprehensionBranch {
    primary_name: String,
    secondary_name: Option<String>,
    iteratee: Rc<Expression>,
    condition: Option<Rc<Expression>>,
    position: Position,
}

impl ListComprehensionBranch {
    pub fn new(
        primary_name: impl Into<String>,
        secondary_name: Option<String>,
        iteratee: impl Into<Expression>,
        condition: Option<Expression>,
        position: Position,
    ) -> Self {
        Self {
            primary_name: primary_name.into(),
            secondary_name,
            iteratee: iteratee.into().into(),
            condition: condition.map(From::from),
            position,
        }
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

    pub fn condition(&self) -> Option<&Expression> {
        self.condition.as_deref()
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
