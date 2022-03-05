use super::expression::Expression;
use crate::MapEntry;

#[derive(Clone, Debug, PartialEq)]
pub enum MapElement {
    Insertion(MapEntry),
    Map(Expression),
    Removal(Expression),
}

impl From<MapEntry> for MapElement {
    fn from(entry: MapEntry) -> Self {
        Self::Insertion(entry)
    }
}
