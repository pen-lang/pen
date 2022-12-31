use super::expression::Expression;
use crate::MapEntry;
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub enum MapElement {
    Multiple(Expression),
    Single(MapEntry),
}

impl MapElement {
    pub fn position(&self) -> &Position {
        match self {
            Self::Multiple(expression) => expression.position(),
            Self::Single(entry) => entry.position(),
        }
    }
}

impl From<MapEntry> for MapElement {
    fn from(entry: MapEntry) -> Self {
        Self::Single(entry)
    }
}
