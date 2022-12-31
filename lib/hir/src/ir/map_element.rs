use super::{expression::Expression, map_entry::MapEntry};

#[derive(Clone, Debug, PartialEq)]
pub enum MapElement {
    Multiple(Expression),
    Single(MapEntry),
}

impl From<MapEntry> for MapElement {
    fn from(entry: MapEntry) -> Self {
        Self::Single(entry)
    }
}
