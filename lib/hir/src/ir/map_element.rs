use super::expression::Expression;
use super::map_entry::MapEntry;

#[derive(Clone, Debug, PartialEq)]
pub enum MapElement {
    Insertion(MapEntry),
    Map(Expression),
    Removal(Expression),
}
