use core::fmt;
use position::Position;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Boolean {
    position: Position,
}

impl Boolean {
    pub fn new(position: Position) -> Self {
        Self { position }
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}

impl Display for Boolean {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "boolean")
    }
}
