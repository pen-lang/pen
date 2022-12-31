use super::{expression::Expression, map_entry::MapEntry};

#[derive(Clone, Debug, PartialEq)]
pub enum MapElement {
    Single(MapEntry),
    Multiple(Expression),
}

impl From<MapEntry> for MapElement {
    fn from(entry: MapEntry) -> Self {
        Self::Single(entry)
    }
}
