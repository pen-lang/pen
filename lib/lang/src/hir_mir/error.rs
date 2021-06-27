use crate::{
    hir::*,
    position::Position,
    types::{self, analysis::TypeError},
};
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug, PartialEq)]
pub enum CompileError {
    AnyTypeBranch(Position),
    FunctionExpected(Position),
    ListExpected(Position),
    MirTypeCheck(mir::analysis::TypeCheckError),
    MissingElseBlock(Position),
    RecordElementUnknown(Position),
    RecordElementMissing(Position),
    RecordExpected(Position),
    RecordNotFound(types::Record),
    TypeAnalysis(TypeError),
    TypeNotFound(types::Reference),
    TypeNotInferred(Position),
    TypesNotMatched(Position, Position),
    UnionOrAnyTypeExpected(Position),
    UnreachableCode(Position),
    VariableNotFound(Variable),
    WrongArgumentCount(Position),
}

impl Display for CompileError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::AnyTypeBranch(position) => {
                write!(
                    formatter,
                    "any type cannot be used for downcast\n{}",
                    position
                )
            }
            Self::FunctionExpected(position) => {
                write!(formatter, "function expected\n{}", position)
            }
            Self::ListExpected(position) => {
                write!(formatter, "list expected\n{}", position)
            }
            Self::MirTypeCheck(error) => {
                write!(formatter, "failed to check types in MIR: {}", error)
            }
            Self::MissingElseBlock(position) => {
                write!(
                    formatter,
                    "missing else block in if-type expression\n{}",
                    position
                )
            }
            Self::RecordElementUnknown(position) => {
                write!(formatter, "unknown record deconstruction\n{}", position)
            }
            Self::RecordElementMissing(position) => {
                write!(formatter, "missing record deconstruction\n{}", position)
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
            Self::UnionOrAnyTypeExpected(position) => {
                write!(formatter, "union or any type expected\n{}", position)
            }
            Self::UnreachableCode(position) => {
                write!(formatter, "unreachable code\n{}", position)
            }
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

impl From<TypeError> for CompileError {
    fn from(error: TypeError) -> Self {
        Self::TypeAnalysis(error)
    }
}
