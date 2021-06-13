use super::{type_resolution, CompileError};
use crate::{
    compile::{type_canonicalization, type_equality},
    types::Type,
};
use std::collections::HashMap;

pub fn check_subsumption(
    lower: &Type,
    upper: &Type,
    types: &HashMap<String, Type>,
) -> Result<bool, CompileError> {
    let lower =
        type_canonicalization::canonicalize(&type_resolution::resolve_type(lower, types)?, types)?;
    let upper =
        type_canonicalization::canonicalize(&type_resolution::resolve_type(upper, types)?, types)?;

    Ok(match (&lower, &upper) {
        (_, Type::Any(_)) => true,
        (Type::List(one), Type::List(other)) => {
            check_subsumption(one.element(), other.element(), types)?
        }
        (Type::Union(lower), Type::Union(_)) => {
            check_subsumption(lower.lhs(), &upper, types)?
                && check_subsumption(lower.rhs(), &upper, types)?
        }
        (lower, Type::Union(union)) => {
            check_subsumption(lower, union.lhs(), types)?
                || check_subsumption(lower, union.rhs(), types)?
        }
        _ => type_equality::check_equality(&lower, &upper, types)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{position::Position, types};

    #[test]
    fn check_numbers() {
        assert!(check_subsumption(
            &types::Number::new(Position::dummy()).into(),
            &types::Number::new(Position::dummy()).into(),
            &Default::default()
        )
        .unwrap());
    }

    #[test]
    fn check_number_and_union() {
        assert!(check_subsumption(
            &types::Number::new(Position::dummy()).into(),
            &types::Union::new(
                types::Number::new(Position::dummy()),
                types::None::new(Position::dummy()),
                Position::dummy()
            )
            .into(),
            &Default::default()
        )
        .unwrap());
    }

    #[test]
    fn check_non_canonical_union_and_number() {
        assert!(check_subsumption(
            &types::Union::new(
                types::Number::new(Position::dummy()),
                types::Number::new(Position::dummy()),
                Position::dummy()
            )
            .into(),
            &types::Number::new(Position::dummy()).into(),
            &Default::default()
        )
        .unwrap());
    }
}
