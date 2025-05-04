use crate::ir::*;
use position::{Position, test::PositionFake};

pub trait FunctionDefinitionFake {
    fn fake(name: impl Into<String>, lambda: Lambda, public: bool) -> Self;
}

impl FunctionDefinitionFake for FunctionDefinition {
    fn fake(name: impl Into<String>, lambda: Lambda, public: bool) -> Self {
        Self::new(name, "", lambda, None, public, Position::fake())
    }
}
