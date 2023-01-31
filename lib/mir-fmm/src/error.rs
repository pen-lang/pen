use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug, PartialEq)]
pub enum CompileError {
    FmmBuild(fmm::build::BuildError),
    NestedVariant,
    ReferenceCount(mir::analysis::reference_count::ReferenceCountError),
    TypeCheck(mir::analysis::type_check::TypeCheckError),
    TypeInformationNotFound(mir::types::Type),
    UnboxedRecord,
}

impl Display for CompileError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for CompileError {}

impl From<fmm::build::BuildError> for CompileError {
    fn from(error: fmm::build::BuildError) -> Self {
        Self::FmmBuild(error)
    }
}

impl From<mir::analysis::reference_count::ReferenceCountError> for CompileError {
    fn from(error: mir::analysis::reference_count::ReferenceCountError) -> Self {
        Self::ReferenceCount(error)
    }
}

impl From<mir::analysis::type_check::TypeCheckError> for CompileError {
    fn from(error: mir::analysis::type_check::TypeCheckError) -> Self {
        Self::TypeCheck(error)
    }
}
