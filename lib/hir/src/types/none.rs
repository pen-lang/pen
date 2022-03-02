use core::fmt;
use position::Position;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct None {
    position: Position,
}

impl None {
    pub fn new(position: Position) -> Self {
        Self { position }
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}

impl Display for None {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "none")
    }
}
