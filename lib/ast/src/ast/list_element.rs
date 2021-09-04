use super::expression::Expression;

#[derive(Clone, Debug, PartialEq)]
pub enum ListElement {
    Multiple(Expression),
    Single(Expression),
}
