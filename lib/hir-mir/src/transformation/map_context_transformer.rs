use super::hash_calculation_transformer;
use crate::{context::CompileContext, transformation::equal_operation_transformer, CompileError};
use hir::{analysis::type_comparability_checker, ir::*, types, types::Type};
use position::Position;

pub fn transform(
    context: &CompileContext,
    key_type: &Type,
    value_type: &Type,
    position: &Position,
) -> Result<Expression, CompileError> {
    let configuration = &context.configuration()?.map_type;
    let any_map_type =
        types::Reference::new(configuration.context_type_name.clone(), position.clone());
    let any_type = Type::from(types::Any::new(position.clone()));
    let equal_function_type = Type::from(types::Function::new(
        vec![any_type.clone(), any_type.clone()],
        types::Boolean::new(position.clone()),
        position.clone(),
    ));
    let hash_function_type = Type::from(types::Function::new(
        vec![any_type],
        types::Number::new(position.clone()),
        position.clone(),
    ));

    Ok(Call::new(
        Some(
            types::Function::new(
                vec![
                    equal_function_type.clone(),
                    hash_function_type.clone(),
                    equal_function_type,
                    hash_function_type,
                ],
                any_map_type,
                position.clone(),
            )
            .into(),
        ),
        Variable::new(&configuration.context_function_name, position.clone()),
        [
            equal_operation_transformer::transform_any_function(context, key_type, position)?
                .into(),
            hash_calculation_transformer::transform_any_function(context, key_type, position)?
                .into(),
        ]
        .into_iter()
        .chain(
            if type_comparability_checker::check(value_type, context.types(), context.records())? {
                [
                    equal_operation_transformer::transform_any_function(
                        context, value_type, position,
                    )?
                    .into(),
                    hash_calculation_transformer::transform_any_function(
                        context, value_type, position,
                    )?
                    .into(),
                ]
            } else {
                [
                    compile_fake_equal_function(position).into(),
                    compile_fake_hash_function(position).into(),
                ]
            },
        )
        .collect(),
        position.clone(),
    )
    .into())
}

fn compile_fake_equal_function(position: &Position) -> Lambda {
    Lambda::new(
        vec![
            Argument::new("", types::Any::new(position.clone())),
            Argument::new("", types::Any::new(position.clone())),
        ],
        types::Boolean::new(position.clone()),
        Boolean::new(false, position.clone()),
        position.clone(),
    )
}

fn compile_fake_hash_function(position: &Position) -> Lambda {
    Lambda::new(
        vec![Argument::new("", types::Any::new(position.clone()))],
        types::Number::new(position.clone()),
        Number::new(0.0, position.clone()),
        position.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use position::test::PositionFake;

    #[test]
    fn transform_none_key_and_none_value() {
        insta::assert_debug_snapshot!(transform(
            &CompileContext::dummy(Default::default(), Default::default()),
            &types::None::new(Position::fake()).into(),
            &types::None::new(Position::fake()).into(),
            &Position::fake()
        ));
    }

    #[test]
    fn transform_function_value() {
        insta::assert_debug_snapshot!(transform(
            &CompileContext::dummy(Default::default(), Default::default()),
            &types::None::new(Position::fake()).into(),
            &types::Function::new(vec![], types::None::new(Position::fake()), Position::fake())
                .into(),
            &Position::fake()
        ));
    }
}
