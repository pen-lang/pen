use crate::{
    hir::*,
    position::Position,
    types::{self, analysis::TypeAnalysisError},
};
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug, PartialEq)]
pub enum CompileError {
    FunctionExpected(Position),
    ListExpected(Position),
    MirTypeCheck(mir::analysis::TypeCheckError),
    RecordElementUnknown(Position),
    RecordElementMissing(Position),
    RecordExpected(Position),
    RecordNotFound(types::Record),
    TypeAnalysis(TypeAnalysisError),
    TypeNotFound(types::Reference),
    TypeNotInferred(Position),
    TypesNotMatched(Position, Position),
    VariableNotFound(Variable),
    WrongArgumentCount(Position),
}

impl Display for CompileError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::FunctionExpected(position) => {
                write!(formatter, "function expected\n{}", position)
            }
            Self::ListExpected(position) => {
                write!(formatter, "list expected\n{}", position)
            }
            Self::MirTypeCheck(error) => {
                write!(formatter, "failed to check types in MIR: {}", error)
            }
            Self::RecordElementUnknown(position) => {
                write!(formatter, "unknown record element\n{}", position)
            }
            Self::RecordElementMissing(position) => {
                write!(formatter, "missing record element\n{}", position)
            }
            Self::RecordExpected(position) => {
                write!(formatter, "record expected\n{}", position)
            }
            Self::RecordNotFound(record) => write!(
                formatter,
                "record type \"{}\" not found\n{}",
                record.name(),
                record.position()
            ),
            Self::TypeAnalysis(error) => write!(formatter, "{}", error),
            Self::TypeNotFound(reference) => write!(
                formatter,
                "type \"{}\" not found\n{}",
                reference.name(),
                reference.position()
            ),
            Self::TypeNotInferred(position) => {
                write!(formatter, "type not inferred\n{}", position)
            }
            Self::TypesNotMatched(lhs_position, rhs_position) => write!(
                formatter,
                "types not matched\n{}\n{}",
                lhs_position, rhs_position
            ),
            Self::VariableNotFound(variable) => write!(
                formatter,
                "variable \"{}\" not found\n{}",
                variable.name(),
                variable.position()
            ),
            Self::WrongArgumentCount(position) => {
                write!(
                    formatter,
                    "wrong number of arguments in function call\n{}",
                    position
                )
            }
        }
    }
}

impl Error for CompileError {}

impl From<mir::analysis::TypeCheckError> for CompileError {
    fn from(error: mir::analysis::TypeCheckError) -> Self {
        Self::MirTypeCheck(error)
    }
}

impl From<TypeAnalysisError> for CompileError {
    fn from(error: TypeAnalysisError) -> Self {
        Self::TypeAnalysis(error)
    }
}
