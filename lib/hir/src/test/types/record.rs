use crate::types;
use position::{test::PositionFake, Position};

pub trait RecordFake {
    fn fake(name: impl Into<String>) -> Self;
}

impl RecordFake for types::Record {
    fn fake(name: impl Into<String>) -> Self {
        let name = name.into();

        Self::new(name.clone(), name, Position::fake())
    }
}
