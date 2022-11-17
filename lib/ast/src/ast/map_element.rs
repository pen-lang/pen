use super::expression::Expression;
use crate::MapEntry;
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub enum MapElement {
    Insertion(MapEntry),
    Map(Expression),
}

impl MapElement {
    pub fn position(&self) -> &Position {
        match self {
            Self::Insertion(entry) => entry.position(),
            Self::Map(expression) => expression.position(),
        }
    }
}

impl From<MapEntry> for MapElement {
    fn from(entry: MapEntry) -> Self {
        Self::Insertion(entry)
    }
}
