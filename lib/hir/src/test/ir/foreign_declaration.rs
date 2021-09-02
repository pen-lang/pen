use crate::{ir::*, types::Type};
use position::{test::PositionFake, Position};

pub trait ForeignDeclarationFake {
    fn fake(name: impl Into<String>, type_: impl Into<Type>) -> Self;
}

impl ForeignDeclarationFake for ForeignDeclaration {
    fn fake(name: impl Into<String>, type_: impl Into<Type>) -> Self {
        Self::new(name, "", CallingConvention::C, type_, Position::fake())
    }
}
