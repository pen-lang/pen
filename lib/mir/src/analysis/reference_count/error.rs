use crate::ir::*;
use std::{collections::BTreeMap, error::Error, fmt::Display};

#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceCountError {
    ExpressionNotSupported(Expression),
    InvalidLocalVariable(String, isize),
    InvalidLocalVariables(BTreeMap<String, isize>),
    UnmatchedVariables(BTreeMap<String, isize>, BTreeMap<String, isize>),
}

impl Display for ReferenceCountError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(formatter, "{:#?}", self)
    }
}

impl Error for ReferenceCountError {}
