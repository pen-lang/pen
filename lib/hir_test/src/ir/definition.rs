use crate::position::position;
use hir::ir::*;

pub trait FakeDefinition {
    fn fake(name: impl Into<String>, lambda: Lambda, public: bool) -> Self;
}

impl FakeDefinition for Definition {
    fn fake(name: impl Into<String>, lambda: Lambda, public: bool) -> Self {
        Self::new(name, "", lambda, false, public, position())
    }
}
