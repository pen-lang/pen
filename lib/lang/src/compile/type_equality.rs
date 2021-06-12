use super::{type_resolution, CompileError};
use crate::types::Type;
use std::collections::HashMap;

pub fn equal_types(
    one: &Type,
    other: &Type,
    types: &HashMap<String, Type>,
) -> Result<bool, CompileError> {
    Ok(
        match (
            type_resolution::resolve_type(one, types)?,
            type_resolution::resolve_type(other, types)?,
        ) {
            (Type::Function(one), Type::Function(other)) => {
                one.arguments().len() == other.arguments().len()
                    && one
                        .arguments()
                        .iter()
                        .zip(other.arguments())
                        .map(|(one, other)| equal_types(one, other, types))
                        .collect::<Result<Vec<_>, _>>()?
                        .iter()
                        .all(|&ok| ok)
                    && equal_types(one.result(), other.result(), types)?
            }
            (Type::List(one), Type::List(other)) => {
                equal_types(one.element(), other.element(), types)?
            }
            (Type::Union(one), Type::Union(other)) => {
                equal_types(one.lhs(), other.lhs(), types)?
                    && equal_types(one.rhs(), other.rhs(), types)?
            }
            (Type::Any(_), Type::Any(_))
            | (Type::Boolean(_), Type::Boolean(_))
            | (Type::None(_), Type::None(_))
            | (Type::Number(_), Type::Number(_))
            | (Type::Record(_), Type::Record(_))
            | (Type::String(_), Type::String(_)) => true,
            (Type::Reference(_), _) | (_, Type::Reference(_)) => unreachable!(),
            _ => {
                return Err(CompileError::TypesNotMatched(
                    one.position().clone(),
                    other.position().clone(),
                ))
            }
        },
    )
}
