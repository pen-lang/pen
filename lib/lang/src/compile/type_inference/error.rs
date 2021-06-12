use crate::{position::Position, types};
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    sync::Arc,
};

#[derive(Clone, Debug, PartialEq)]
pub enum TypeInferenceError {
    TypeNotFound(Arc<types::Reference>),
    TypesNotMatched(Arc<Position>, Arc<Position>),
}

impl Error for TypeInferenceError {}

impl Display for TypeInferenceError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::TypeNotFound(reference) => write!(
                formatter,
                "type \"{}\" not found\n{}",
                reference.name(),
                reference.position()
            ),
            Self::TypesNotMatched(lhs_source_information, rhs_source_information) => write!(
                formatter,
                "types not matched\n{}\n{}",
                lhs_source_information, rhs_source_information
            ),
        }
    }
}
