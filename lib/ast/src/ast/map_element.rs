use super::expression::Expression;
use crate::MapEntry;

#[derive(Clone, Debug, PartialEq)]
pub enum MapElement {
    Insertion(MapEntry),
    Map(Expression),
    Removal(Expression),
}
