use super::expression::Expression;
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct ListComprehensionBranch {
    names: Vec<String>,
    iteratees: Vec<Expression>,
    condition: Option<Rc<Expression>>,
    position: Position,
}

impl ListComprehensionBranch {
    pub fn new(
        names: Vec<String>,
        iteratees: Vec<Expression>,
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
