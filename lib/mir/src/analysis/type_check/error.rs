use crate::{
    ir::*,
    types::{self, Type},
};
use std::{error::Error, fmt::Display};

#[derive(Clone, Debug, PartialEq)]
pub enum TypeCheckError {
    DuplicateFunctionNames(String),
    DuplicateTypeNames(String),
    FieldIndexOutOfBounds(RecordField),
    ForeignDefinitionNotFound(ForeignDefinition),
    FunctionExpected(Expression),
    NoAlternativeFound(Case),
    TypeNotFound(types::Record),
    TypesNotMatched(Type, Type),
    VariableNotFound(Variable),
    VariantInVariant(Variant),
    WrongArgumentCount(Call),
    WrongFieldCount(Record),
}

impl Display for TypeCheckError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(formatter, "{:#?}", self)
    }
}

impl Error for TypeCheckError {}
