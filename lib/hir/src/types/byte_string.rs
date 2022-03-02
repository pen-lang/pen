use core::fmt;
use position::Position;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ByteString {
    position: Position,
}

impl ByteString {
    pub fn new(position: Position) -> Self {
        Self { position }
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}

impl Display for ByteString {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "boolean")
    }
}
