use super::{collection_type, hash_calculation};
use crate::{context::CompileContext, transformation::equal_operation, CompileError};
use hir::{analysis::type_comparability_checker, ir::*, types, types::Type};
use position::Position;

pub fn transform(
    context: &CompileContext,
    key_type: &Type,
    value_type: &Type,
    position: &Position,
) -> Result<Expression, CompileError> {
    let configuration = &context.configuration()?.map_type;
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
    let context_type = collection_type::transform_map_context(context, position)?;

    // This thunk is lifted as a global function definition by lambda lifting later.
    // TODO Define only one map context per type.
    Ok(Call::new(
        Some(types::Function::new(vec![], context_type.clone(), position.clone()).into()),
        Thunk::new(
            Some(context_type.clone().into()),
            Call::new(
                Some(
                    types::Function::new(
                        vec![
                            equal_function_type.clone(),
                            hash_function_type.clone(),
                            equal_function_type,
                            hash_function_type,
                        ],
                        context_type.clone(),
                        position.clone(),
                    )
                    .into(),
                ),
                Variable::new(&configuration.context_function_name, position.clone()),
                [
                    equal_operation::transform_any_function(context, key_type, position)?.into(),
                    hash_calculation::transform_any_function(context, key_type, position)?.into(),
                ]
                .into_iter()
                .chain(
                    if type_comparability_checker::check(
                        value_type,
                        context.types(),
                        context.records(),
                    )? {
                        [
                            equal_operation::transform_any_function(context, value_type, position)?
                                .into(),
                            hash_calculation::transform_any_function(
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
            ),
            position.clone(),
        ),
        vec![],
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
