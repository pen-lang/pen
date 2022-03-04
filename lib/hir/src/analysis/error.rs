use crate::{ir::*, types::*};
use position::Position;
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug, PartialEq)]
pub enum AnalysisError {
    FunctionExpected(Position),
    ListExpected(Position),
    RecordExpected(Position),
    RecordNotFound(Record),
    TypeNotFound(Reference),
    TypeNotInferred(Position),
    UnionExpected(Position),
    UnknownRecordField(Position),
    UnreachableCode(Position),
    VariableNotFound(Variable),
}

impl Display for AnalysisError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::FunctionExpected(position) => {
                write!(formatter, "function expected\n{}", position)
            }
            Self::ListExpected(position) => {
                write!(formatter, "list expected\n{}", position)
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
            Self::TypeNotFound(reference) => write!(
                formatter,
                "type \"{}\" not found\n{}",
                reference.name(),
                reference.position()
            ),
            Self::TypeNotInferred(position) => {
                write!(formatter, "type not inferred\n{}", position)
            }
            Self::UnionExpected(position) => {
                write!(formatter, "union expected\n{}", position)
            }
            Self::UnknownRecordField(position) => {
                write!(formatter, "unknown record field\n{}", position)
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
        }
    }
}

impl Error for AnalysisError {}
