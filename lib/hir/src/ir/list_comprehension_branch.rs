use super::expression::Expression;
use crate::types::Type;
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct ListComprehensionBranch {
    type_: Option<Type>,
    names: Vec<String>,
    iteratees: Vec<Expression>,
    condition: Option<Rc<Expression>>,
    position: Position,
}

impl ListComprehensionBranch {
    pub fn new(
        type_: Option<Type>,
        names: Vec<String>,
        iteratees: Vec<Expression>,
        condition: Option<Expression>,
        position: Position,
    ) -> Self {
        Self {
            type_,
            names,
            iteratees,
            condition: condition.map(From::from),
            position,
        }
    }

    pub fn type_(&self) -> Option<&Type> {
        self.type_.as_ref()
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }

    pub fn iteratees(&self) -> &[Expression] {
        &self.iteratees
    }

    pub fn condition(&self) -> Option<&Expression> {
        self.condition.as_deref()
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
