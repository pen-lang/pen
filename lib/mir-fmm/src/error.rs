use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug, PartialEq)]
pub enum CompileError {
    FmmBuild(fmm::build::BuildError),
    HeapReuse(mir::analysis::ReuseError),
    NestedVariant,
    ReferenceCount(mir::analysis::ReferenceCountError),
    TypeCheck(mir::analysis::TypeCheckError),
    UnboxedRecord,
}

impl Display for CompileError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

impl Error for CompileError {}

impl From<fmm::build::BuildError> for CompileError {
    fn from(error: fmm::build::BuildError) -> Self {
        Self::FmmBuild(error)
    }
}

impl From<mir::analysis::ReferenceCountError> for CompileError {
    fn from(error: mir::analysis::ReferenceCountError) -> Self {
        Self::ReferenceCount(error)
    }
}

impl From<mir::analysis::ReuseError> for CompileError {
    fn from(error: mir::analysis::ReuseError) -> Self {
        Self::HeapReuse(error)
    }
}

impl From<mir::analysis::TypeCheckError> for CompileError {
    fn from(error: mir::analysis::TypeCheckError) -> Self {
        Self::TypeCheck(error)
    }
}
