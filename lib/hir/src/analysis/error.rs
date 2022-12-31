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
    ArgumentCount(Position),
    BuiltInFunctionNotCalled(Position),
    CollectionExpected(Position, Type),
    DuplicateFunctionNames(Position, Position),
    DuplicateTypeNames(Position, Position),
    ElementNameNotDefined(Position),
    ErrorTypeUndefined,
    FunctionExpected(Position, Type),
    ImpossibleRecord(Position),
    InvalidAdditionOperand(Position),
    InvalidTryOperation(Position),
    KeyNameNotDefined(Position),
    ListComprehensionIterateeCount(Position),
    ListExpected(Position, Type),
    MapExpected(Position, Type),
    MissingElseBlock(Position),
    RecordExpected(Position, Type),
    RecordFieldNotFound(String, Position),
    RecordFieldPrivate(Position),
    RecordFieldUnknown(Position),
    RecordNotFound(Record),
    RecursiveTypeAlias(Position),
    SpawnedFunctionArguments(Position),
    TryOperationInList(Position),
    TypeNotFound(Reference),
    TypeNotInferred(Position),
    TypeNotComparable(Position, Type),
    TypesNotMatched {
        found: (Position, Type),
        expected: (Position, Type),
    },
    UnionExpected(Position, Type),
    UnknownRecordField(Position),
    UnreachableCode(Position),
    UnusedErrorValue(Position),
    ValueNameNotDefined(Position),
    VariableNotFound(Variable),
    VariantExpected(Position, Type),
}

impl AnalysisError {
    fn format_type(type_: &Type) -> String {
        format!("`{}`", type_formatter::format(type_))
    }

    fn format_found_type_message(position: &Position, type_: &Type) -> String {
        position::format_message(position, &format!("found {}", Self::format_type(type_)))
    }

    fn format_expected_type_message(position: &Position, type_: &Type) -> String {
        position::format_message(position, &format!("expected {}", Self::format_type(type_)))
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
            Self::ArgumentCount(position) => {
                write!(
                    formatter,
                    "wrong number of arguments in function call\n{}",
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
            Self::CollectionExpected(position, type_) => {
                write!(
                    formatter,
                    "list or map expected\n{}",
                    Self::format_found_type_message(position, type_)
                )
            }
            Self::DuplicateFunctionNames(one, other) => {
                write!(formatter, "duplicate function names\n{}\n{}", one, other)
            }
            Self::DuplicateTypeNames(one, other) => {
                write!(formatter, "duplicate type names\n{}\n{}", one, other)
            }
            Self::ElementNameNotDefined(position) => {
                write!(formatter, "element name not defined\n{}", position)
            }
            Self::ErrorTypeUndefined => {
                write!(formatter, "error type undefined")
            }
            Self::FunctionExpected(position, type_) => {
                write!(
                    formatter,
                    "function expected\n{}",
                    Self::format_found_type_message(position, type_)
                )
            }
            Self::ImpossibleRecord(position) => {
                write!(
                    formatter,
                    "record construction dependent on itself\n{}",
                    position
                )
            }
            Self::InvalidAdditionOperand(position) => {
                write!(
                    formatter,
                    "addition operands must be numbers or strings\n{}",
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
            Self::KeyNameNotDefined(position) => {
                write!(formatter, "key name not defined\n{}", position)
            }
            Self::ListComprehensionIterateeCount(position) => {
                write!(
                    formatter,
                    "unmatched iteratee count in list comprehension\n{}",
                    position
                )
            }
            Self::ListExpected(position, type_) => {
                write!(
                    formatter,
                    "list expected\n{}",
                    Self::format_found_type_message(position, type_)
                )
            }
            Self::MapExpected(position, type_) => {
                write!(
                    formatter,
                    "map expected\n{}",
                    Self::format_found_type_message(position, type_)
                )
            }
            Self::MissingElseBlock(position) => {
                write!(
                    formatter,
                    "missing else block in if-type expression\n{}",
                    position
                )
            }
            Self::RecordExpected(position, type_) => {
                write!(
                    formatter,
                    "record expected\n{}",
                    Self::format_found_type_message(position, type_)
                )
            }
            Self::RecordFieldNotFound(name, position) => {
                write!(formatter, "missing record field: {}\n{}", name, position)
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
            Self::TypeNotComparable(position, type_) => {
                write!(
                    formatter,
                    "type not comparable\n{}",
                    position::format_message(
                        position,
                        &format!(
                            "{} might include function, {}, or {} types",
                            Self::format_type(type_),
                            Self::format_type(&types::Error::new(position.clone()).into()),
                            Self::format_type(&types::Any::new(position.clone()).into()),
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
            Self::TypesNotMatched {
                found: (found_position, found_type),
                expected: (expected_position, expected_type),
            } => write!(
                formatter,
                "types not matched\n{}\n{}",
                Self::format_found_type_message(found_position, found_type),
                Self::format_expected_type_message(expected_position, expected_type),
            ),
            Self::UnionExpected(position, type_) => {
                write!(
                    formatter,
                    "union type expected\n{}",
                    Self::format_found_type_message(position, type_)
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
            Self::ValueNameNotDefined(position) => {
                write!(formatter, "value name not defined\n{}", position)
            }
            Self::VariableNotFound(variable) => write!(
                formatter,
                "variable \"{}\" not found\n{}",
                variable.name(),
                variable.position()
            ),
            Self::VariantExpected(position, type_) => {
                write!(
                    formatter,
                    "union or any type expected\n{}",
                    Self::format_found_type_message(position, type_)
                )
            }
        }
    }
}

impl Error for AnalysisError {}
