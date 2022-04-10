use crate::{context::CompileContext, CompileError};
use hir::{
    analysis::{
        type_canonicalizer, type_equality_checker, type_subsumption_checker, AnalysisError,
    },
    ir::*,
    types::Type,
};

const VALUE_NAME: &str = "$value";

pub fn compile(
    context: &CompileContext,
    from: &Type,
    to: &Type,
    expression: &Expression,
) -> Result<Expression, CompileError> {
    let position = expression.position();
    let from = type_canonicalizer::canonicalize(from, context.types())?;

    if !from.is_variant() {
        return Err(AnalysisError::VariantExpected(expression.position().clone()).into());
    } else if !type_subsumption_checker::check(to, &from, context.types())? {
        return Err(
            AnalysisError::TypesNotMatched(to.position().clone(), from.position().clone()).into(),
        );
    }

    Ok(
        if type_equality_checker::check(&from, to, context.types())? {
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
            &CompileContext::dummy(Default::default(), Default::default()),
            from,
            to,
            expression,
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
            Err(AnalysisError::TypesNotMatched(Position::fake(), Position::fake()).into())
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
            Err(AnalysisError::VariantExpected(Position::fake()).into())
        );
    }
}
