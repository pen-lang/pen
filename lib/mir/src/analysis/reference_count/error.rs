use crate::ir::*;
use fnv::FnvHashMap;
use std::{error::Error, fmt::Display};

#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceCountError {
    ExpressionNotSupported(Expression),
    InvalidLocalVariable(String, isize),
    InvalidLocalVariables(FnvHashMap<String, isize>),
    UnmatchedVariables(FnvHashMap<String, isize>, FnvHashMap<String, isize>),
}

impl Display for ReferenceCountError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(formatter, "{self:#?}")
    }
}

impl Error for ReferenceCountError {}
