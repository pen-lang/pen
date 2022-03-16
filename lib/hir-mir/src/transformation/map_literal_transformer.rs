use super::{equal_operation_transformer, hash_calculation_transformer};
use crate::{context::CompileContext, CompileError};
use hir::{
    analysis::type_comparability_checker,
    ir::*,
    types::{self, Type},
};
use position::Position;

pub fn transform(context: &CompileContext, map: &Map) -> Result<Expression, CompileError> {
    transform_map(
        context,
        map.key_type(),
        map.value_type(),
        map.elements(),
        map.position(),
    )
}

fn transform_map(
    context: &CompileContext,
    key_type: &Type,
    value_type: &Type,
    elements: &[MapElement],
    position: &Position,
) -> Result<Expression, CompileError> {
    let configuration = &context.configuration()?.map_type;
    let any_map_type = types::Reference::new(configuration.map_type_name.clone(), position.clone());
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

    Ok(match elements {
        [] => Call::new(
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
            Variable::new(&configuration.empty_function_name, position.clone()),
            [
                equal_operation_transformer::transform_any_function(context, key_type, position)?
                    .into(),
                hash_calculation_transformer::transform_any_function(context, key_type, position)?
                    .into(),
            ]
            .into_iter()
            .chain(
                if type_comparability_checker::check(
                    value_type,
                    context.types(),
                    context.records(),
                )? {
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
        .into(),
        // TODO Optimize cases where only a single element of MapElement::Map
        // exists when we pass context functions dynamically.
        [element, ..] => {
            let rest_expression =
                transform_map(context, key_type, value_type, &elements[1..], position)?;

            match element {
                MapElement::Insertion(entry) => Call::new(
                    Some(
                        types::Function::new(
                            vec![
                                any_map_type.clone().into(),
                                types::Any::new(position.clone()).into(),
                                types::Any::new(position.clone()).into(),
                            ],
                            any_map_type,
                            position.clone(),
                        )
                        .into(),
                    ),
                    Variable::new(&configuration.set_function_name, position.clone()),
                    vec![
                        rest_expression,
                        TypeCoercion::new(
                            key_type.clone(),
                            types::Any::new(position.clone()),
                            entry.key().clone(),
                            position.clone(),
                        )
                        .into(),
                        TypeCoercion::new(
                            value_type.clone(),
                            types::Any::new(position.clone()),
                            entry.value().clone(),
                            position.clone(),
                        )
                        .into(),
                    ],
                    position.clone(),
                )
                .into(),
                MapElement::Map(expression) => Call::new(
                    Some(
                        types::Function::new(
                            vec![any_map_type.clone().into(), any_map_type.clone().into()],
                            any_map_type,
                            position.clone(),
                        )
                        .into(),
                    ),
                    Variable::new(&configuration.merge_function_name, position.clone()),
                    vec![expression.clone(), rest_expression],
                    position.clone(),
                )
                .into(),
                MapElement::Removal(key) => Call::new(
                    Some(
                        types::Function::new(
                            vec![
                                any_map_type.clone().into(),
                                types::Any::new(position.clone()).into(),
                            ],
                            any_map_type,
                            position.clone(),
                        )
                        .into(),
                    ),
                    Variable::new(&configuration.delete_function_name, position.clone()),
                    vec![
                        rest_expression,
                        TypeCoercion::new(
                            key_type.clone(),
                            types::Any::new(position.clone()),
                            key.clone(),
                            position.clone(),
                        )
                        .into(),
                    ],
                    position.clone(),
                )
                .into(),
            }
        }
    })
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
    fn transform_empty_map() {
        insta::assert_debug_snapshot!(transform(
            &CompileContext::dummy(Default::default(), Default::default()),
            &Map::new(
                types::None::new(Position::fake()),
                types::None::new(Position::fake()),
                vec![],
                Position::fake()
            ),
        ));
    }

    #[test]
    fn transform_empty_map_with_function_value() {
        insta::assert_debug_snapshot!(transform(
            &CompileContext::dummy(Default::default(), Default::default()),
            &Map::new(
                types::None::new(Position::fake()),
                types::Function::new(vec![], types::None::new(Position::fake()), Position::fake()),
                vec![],
                Position::fake()
            ),
        ));
    }

    #[test]
    fn transform_map_with_entry() {
        insta::assert_debug_snapshot!(transform(
            &CompileContext::dummy(Default::default(), Default::default()),
            &Map::new(
                types::None::new(Position::fake()),
                types::None::new(Position::fake()),
                vec![MapEntry::new(
                    None::new(Position::fake()),
                    None::new(Position::fake()),
                    Position::fake()
                )
                .into()],
                Position::fake()
            ),
        ));
    }
}
