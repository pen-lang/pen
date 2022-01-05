use crate::{type_context::TypeContext, CompileError};
use hir::{
    analysis::types::{type_canonicalizer, type_equality_checker, type_subsumption_checker},
    ir::*,
    types::Type,
};

const VALUE_NAME: &str = "$value";

pub fn compile(
    from: &Type,
    to: &Type,
    expression: &Expression,
    type_context: &TypeContext,
) -> Result<Expression, CompileError> {
    let position = expression.position();
    let from = type_canonicalizer::canonicalize(from, type_context.types())?;

    if !from.is_union() && !from.is_any() {
        return Err(CompileError::UnionOrAnyTypeExpected(
            expression.position().clone(),
        ));
    } else if !type_subsumption_checker::check(to, &from, type_context.types())? {
        return Err(CompileError::TypesNotMatched(
            to.position().clone(),
            from.position().clone(),
        ));
    }

    Ok(
        if type_equality_checker::check(&from, to, type_context.types())? {
            expression.clone()
        } else {
            IfType::new(
                VALUE_NAME,
                expression.clone(),
                vec![IfTypeBranch::new(
                    to.clone(),
                    Variable::new(VALUE_NAME, position.clone()),
                )],
                None,
                position.clone(),
            )
            .into()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use hir::types;
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    fn downcast(
        from: &Type,
        to: &Type,
        expression: &Expression,
    ) -> Result<Expression, CompileError> {
        compile(
            from,
            to,
            expression,
            &TypeContext::dummy(Default::default(), Default::default()),
        )
    }

    #[test]
    fn downcast_any_to_none() {
        assert_eq!(
            downcast(
                &types::Any::new(Position::fake()).into(),
                &types::None::new(Position::fake()).into(),
                &Variable::new("x", Position::fake()).into(),
            ),
            Ok(IfType::new(
                VALUE_NAME,
                Variable::new("x", Position::fake()),
                vec![IfTypeBranch::new(
                    types::None::new(Position::fake()),
                    Variable::new(VALUE_NAME, Position::fake()),
                )],
                None,
                Position::fake(),
            )
            .into())
        );
    }

    #[test]
    fn downcast_any_to_any() {
        assert_eq!(
            downcast(
                &types::Any::new(Position::fake()).into(),
                &types::Any::new(Position::fake()).into(),
                &Variable::new("x", Position::fake()).into(),
            ),
            Ok(Variable::new("x", Position::fake()).into())
        );
    }

    #[test]
    fn downcast_union_to_none() {
        assert_eq!(
            downcast(
                &types::Union::new(
                    types::None::new(Position::fake()),
                    types::Number::new(Position::fake()),
                    Position::fake()
                )
                .into(),
                &types::None::new(Position::fake()).into(),
                &Variable::new("x", Position::fake()).into(),
            ),
            Ok(IfType::new(
                VALUE_NAME,
                Variable::new("x", Position::fake()),
                vec![IfTypeBranch::new(
                    types::None::new(Position::fake()),
                    Variable::new(VALUE_NAME, Position::fake()),
                )],
                None,
                Position::fake(),
            )
            .into())
        );
    }

    #[test]
    fn fail_to_downcast_union_to_any() {
        assert_eq!(
            downcast(
                &types::Union::new(
                    types::None::new(Position::fake()),
                    types::Number::new(Position::fake()),
                    Position::fake()
                )
                .into(),
                &types::Any::new(Position::fake()).into(),
                &Variable::new("x", Position::fake()).into(),
            ),
            Err(CompileError::TypesNotMatched(
                Position::fake(),
                Position::fake()
            ))
        );
    }

    #[test]
    fn fail_to_downcast_non_union() {
        assert_eq!(
            downcast(
                &types::None::new(Position::fake()).into(),
                &types::None::new(Position::fake()).into(),
                &None::new(Position::fake()).into(),
            ),
            Err(CompileError::UnionOrAnyTypeExpected(Position::fake()))
        );
    }
}
