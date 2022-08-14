use super::type_formatter;
use crate::{
    ir::*,
    types::{self, *},
};
use position::Position;
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnalysisError {
    AnyTypeBranch(Position),
    BuiltInFunctionNotCalled(Position),
    CollectionExpected(Type),
    DuplicateFunctionNames(Position, Position),
    DuplicateTypeNames(Position, Position),
    ErrorTypeUndefined,
    FunctionExpected(Type),
    ImpossibleRecord(Position),
    InvalidTryOperation(Position),
    ListExpected(Type),
    MapExpected(Type),
    MissingElseBlock(Position),
    RecordExpected(Type),
    RecordFieldMissing(Position),
    RecordFieldPrivate(Position),
    RecordFieldUnknown(Position),
    RecordNotFound(Record),
    RecursiveTypeAlias(Position),
    SpawnedFunctionArguments(Position),
    TryOperationInList(Position),
    TypeNotFound(Reference),
    TypeNotInferred(Position),
    TypeNotComparable(Type),
    TypesNotMatched(Type, Type),
    UnionExpected(Type),
    UnknownRecordField(Position),
    UnreachableCode(Position),
    UnusedErrorValue(Position),
    VariableNotFound(Variable),
    VariantExpected(Type),
    WrongArgumentCount(Position),
}

impl AnalysisError {
    fn format_type(type_: &Type) -> String {
        format!("`{}`", type_formatter::format(type_))
    }

    fn format_found_type_message(type_: &Type) -> String {
        position::format_message(
            type_.position(),
            &format!("found {}", Self::format_type(type_)),
        )
    }

    fn format_expected_type_message(type_: &Type) -> String {
        position::format_message(
            type_.position(),
            &format!("expected {}", Self::format_type(type_)),
        )
    }
}

impl Display for AnalysisError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::AnyTypeBranch(position) => {
                write!(
                    formatter,
                    "any type cannot be used for downcast\n{}",
                    position
                )
            }
            Self::BuiltInFunctionNotCalled(position) => {
                write!(
                    formatter,
                    "built-in function must be called directly\n{}",
                    position
                )
            }
            Self::CollectionExpected(type_) => {
                write!(
                    formatter,
                    "list or map expected\n{}",
                    Self::format_found_type_message(type_)
                )
            }
            Self::DuplicateFunctionNames(one, other) => {
                write!(formatter, "duplicate function names\n{}\n{}", one, other)
            }
            Self::DuplicateTypeNames(one, other) => {
                write!(formatter, "duplicate type names\n{}\n{}", one, other)
            }
            Self::ErrorTypeUndefined => {
                write!(formatter, "error type undefined")
            }
            Self::FunctionExpected(type_) => {
                write!(
                    formatter,
                    "function expected\n{}",
                    Self::format_found_type_message(type_)
                )
            }
            Self::ImpossibleRecord(position) => {
                write!(
                    formatter,
                    "record construction dependent on itself\n{}",
                    position
                )
            }
            Self::InvalidTryOperation(position) => {
                write!(
                    formatter,
                    "try operation cannot be used in function not returning error\n{}",
                    position
                )
            }
            Self::ListExpected(type_) => {
                write!(
                    formatter,
                    "list expected\n{}",
                    Self::format_found_type_message(type_)
                )
            }
            Self::MapExpected(type_) => {
                write!(
                    formatter,
                    "map expected\n{}",
                    Self::format_found_type_message(type_)
                )
            }
            Self::MissingElseBlock(position) => {
                write!(
                    formatter,
                    "missing else block in if-type expression\n{}",
                    position
                )
            }
            Self::RecordExpected(type_) => {
                write!(
                    formatter,
                    "record expected\n{}",
                    Self::format_found_type_message(type_)
                )
            }
            Self::RecordFieldMissing(position) => {
                write!(formatter, "missing record field\n{}", position)
            }
            Self::RecordFieldPrivate(position) => {
                write!(formatter, "private record field\n{}", position)
            }
            Self::RecordFieldUnknown(position) => {
                write!(formatter, "unknown record field\n{}", position)
            }
            Self::RecordNotFound(record) => write!(
                formatter,
                "record type \"{}\" not found\n{}",
                record.name(),
                record.position()
            ),
            Self::RecursiveTypeAlias(position) => {
                write!(formatter, "recursive type alias\n{}", position)
            }
            Self::SpawnedFunctionArguments(position) => {
                write!(
                    formatter,
                    "function passed to go built-in function cannot have any argument\n{}",
                    position
                )
            }
            Self::TryOperationInList(position) => {
                write!(
                    formatter,
                    "try operation not allowed in list literal\n{}",
                    position
                )
            }
            Self::TypeNotComparable(type_) => {
                write!(
                    formatter,
                    "type not comparable\n{}",
                    position::format_message(
                        type_.position(),
                        &format!(
                            "{} might include function, {}, or {} types",
                            Self::format_type(type_),
                            Self::format_type(&types::Error::new(type_.position().clone()).into()),
                            Self::format_type(&types::Any::new(type_.position().clone()).into()),
                        ),
                    )
                )
            }
            Self::TypeNotFound(reference) => write!(
                formatter,
                "type \"{}\" not found\n{}",
                reference.name(),
                reference.position()
            ),
            Self::TypeNotInferred(position) => {
                write!(formatter, "type not inferred\n{}", position)
            }
            Self::TypesNotMatched(lower, upper) => write!(
                formatter,
                "types not matched\n{}\n{}",
                Self::format_found_type_message(lower),
                Self::format_expected_type_message(upper),
            ),
            Self::UnionExpected(type_) => {
                write!(
                    formatter,
                    "union type expected\n{}",
                    Self::format_found_type_message(type_)
                )
            }
            Self::UnknownRecordField(position) => {
                write!(formatter, "unknown record field\n{}", position)
            }
            Self::UnreachableCode(position) => {
                write!(formatter, "unreachable code\n{}", position)
            }
            Self::UnusedErrorValue(position) => {
                write!(formatter, "unused error value\n{}", position)
            }
            Self::VariableNotFound(variable) => write!(
                formatter,
                "variable \"{}\" not found\n{}",
                variable.name(),
                variable.position()
            ),
            Self::VariantExpected(type_) => {
                write!(
                    formatter,
                    "union or any type expected\n{}",
                    Self::format_found_type_message(type_)
                )
            }
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

impl Error for AnalysisError {}
