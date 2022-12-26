use super::{expression::Expression, ListComprehensionIteratee};
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct ListComprehensionBranch {
    names: Vec<String>,
    iteratees: Vec<ListComprehensionIteratee>,
    condition: Option<Rc<Expression>>,
    position: Position,
}

impl ListComprehensionBranch {
    pub fn new(
        names: Vec<String>,
        iteratees: Vec<ListComprehensionIteratee>,
        condition: Option<Expression>,
        position: Position,
    ) -> Self {
        Self {
            names,
            iteratees,
            condition: condition.map(From::from),
            position,
        }
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }

    pub fn iteratees(&self) -> &[ListComprehensionIteratee] {
        &self.iteratees
    }

    pub fn condition(&self) -> Option<&Expression> {
        self.condition.as_deref()
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
