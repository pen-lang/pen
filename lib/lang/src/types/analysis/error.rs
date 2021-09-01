use super::super::*;
use position::Position;
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug, PartialEq)]
pub enum TypeError {
    RecordExpected(Position),
    RecordNotFound(Record),
    TypeNotFound(Reference),
}

impl Display for TypeError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
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
        }
    }
}

impl Error for TypeError {}
