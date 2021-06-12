use super::{type_context::TypeContext, type_resolution, CompileError};
use crate::types::Type;

pub fn equal_types(one: &Type, other: &Type, context: &TypeContext) -> Result<bool, CompileError> {
    Ok(
        match (
            type_resolution::resolve_type(one, context)?,
            type_resolution::resolve_type(other, context)?,
        ) {
            (Type::Function(one), Type::Function(other)) => {
                one.arguments().len() == other.arguments().len()
                    && one
                        .arguments()
                        .iter()
                        .zip(other.arguments())
                        .map(|(one, other)| equal_types(one, other, context))
                        .collect::<Result<Vec<_>, _>>()?
                        .iter()
                        .all(|&ok| ok)
                    && equal_types(one.result(), other.result(), context)?
            }
            (Type::List(one), Type::List(other)) => {
                equal_types(one.element(), other.element(), context)?
            }
            (Type::Union(one), Type::Union(other)) => {
                equal_types(one.lhs(), other.lhs(), context)?
                    && equal_types(one.rhs(), other.rhs(), context)?
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
