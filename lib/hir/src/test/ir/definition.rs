use crate::ir::*;
use position::{test::PositionFake, Position};

pub trait DefinitionFake {
    fn fake(name: impl Into<String>, lambda: Lambda, public: bool) -> Self;
}

impl DefinitionFake for Definition {
    fn fake(name: impl Into<String>, lambda: Lambda, public: bool) -> Self {
        Self::new(name, "", lambda, None, public, Position::fake())
    }
}
