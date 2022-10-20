use super::Type;
use position::Position;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Map(Rc<MapInner>);

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
struct MapInner {
    key: Type,
    value: Type,
    position: Position,
}

impl Map {
    pub fn new(key: impl Into<Type>, value: impl Into<Type>, position: Position) -> Self {
        Self(
            MapInner {
                key: (key.into()),
                value: (value.into()),
                position,
            }
            .into(),
        )
    }

    pub fn key(&self) -> &Type {
        &self.0.key
    }

    pub fn value(&self) -> &Type {
        &self.0.value
    }

    pub fn position(&self) -> &Position {
        &self.0.position
    }

    pub fn set_position(&self, position: Position) -> Self {
        Self(
            MapInner {
                key: (self.0.key.clone()),
                value: (self.0.value.clone()),
                position,
            }
            .into(),
        )
    }
}
