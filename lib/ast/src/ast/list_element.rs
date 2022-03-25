use super::expression::Expression;
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub enum ListElement {
    Multiple(Expression),
    Single(Expression),
}

impl ListElement {
    pub fn position(&self) -> &Position {
        match self {
            Self::Multiple(expression) => expression.position(),
            Self::Single(expression) => expression.position(),
        }
    }
}
