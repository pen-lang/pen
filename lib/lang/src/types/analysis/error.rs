use super::super::*;
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug, PartialEq)]
pub enum TypeAnalysisError {
    RecordNotFound(Record),
    TypeNotFound(Reference),
}

impl Display for TypeAnalysisError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
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
        }
    }
}

impl Error for TypeAnalysisError {}
