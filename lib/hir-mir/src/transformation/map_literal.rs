use super::{collection_type, map_context};
use crate::{context::Context, CompileError};
use hir::{
    ir::*,
    types::{self, Type},
};
use position::Position;

const CONTEXT_VARIABLE_NAME: &str = "$ctx";

pub fn transform(context: &Context, map: &Map) -> Result<Expression, CompileError> {
    let key_type = map.key_type();
    let value_type = map.value_type();
    let position = map.position();

    let map_context_type = collection_type::transform_map_context(context, position)?;

    Ok(Let::new(
        Some(CONTEXT_VARIABLE_NAME.into()),
        Some(map_context_type.clone()),
        map_context::expression::transform(
            context,
            &types::Map::new(key_type.clone(), value_type.clone(), position.clone()),
        )?,
        transform_map(
            context,
            &Variable::new(CONTEXT_VARIABLE_NAME, position.clone()).into(),
            &map_context_type,
            key_type,
            value_type,
            map.elements(),
            position,
        )?,
        position.clone(),
    )
    .into())
}

fn transform_map(
    context: &Context,
    map_context: &Expression,
    map_context_type: &Type,
    key_type: &Type,
    value_type: &Type,
    elements: &[MapElement],
    position: &Position,
) -> Result<Expression, CompileError> {
    let configuration = &context.configuration()?.map_type;
    let any_map_type = collection_type::transform_map(context, position)?;

    Ok(match elements {
        [] => Call::new(
            Some(types::Function::new(vec![], any_map_type, position.clone()).into()),
            Variable::new(&configuration.empty_function_name, position.clone()),
            vec![],
            position.clone(),
        )
        .into(),
        // Optimize cases where only a single element of a spread map exists.
        // This is safe because we pass in context functions dynamically in every map operation.
        [MapElement::Multiple(expression)] => expression.clone(),
        [.., element] => {
            let rest_expression = transform_map(
                context,
                map_context,
                map_context_type,
                key_type,
                value_type,
                &elements[..elements.len() - 1],
                position,
            )?;

            match element {
                MapElement::Single(entry) => Call::new(
                    Some(
                        types::Function::new(
                            vec![
                                map_context_type.clone(),
                                any_map_type.clone(),
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
                        map_context.clone(),
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
                MapElement::Multiple(expression) => Call::new(
                    Some(
                        types::Function::new(
                            vec![
                                map_context_type.clone(),
                                any_map_type.clone(),
                                any_map_type.clone(),
                            ],
                            any_map_type,
                            position.clone(),
                        )
                        .into(),
                    ),
                    Variable::new(&configuration.merge_function_name, position.clone()),
                    vec![map_context.clone(), expression.clone(), rest_expression],
                    position.clone(),
                )
                .into(),
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use position::test::PositionFake;

    #[test]
    fn transform_empty_map() {
        insta::assert_debug_snapshot!(transform(
            &Context::dummy(Default::default(), Default::default()),
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
            &Context::dummy(Default::default(), Default::default()),
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
            &Context::dummy(Default::default(), Default::default()),
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

    #[test]
    fn transform_map_with_2_entries() {
        insta::assert_debug_snapshot!(transform(
            &Context::dummy(Default::default(), Default::default()),
            &Map::new(
                types::None::new(Position::fake()),
                types::None::new(Position::fake()),
                vec![
                    MapEntry::new(
                        Number::new(1.0, Position::fake()),
                        None::new(Position::fake()),
                        Position::fake()
                    )
                    .into(),
                    MapEntry::new(
                        Number::new(2.0, Position::fake()),
                        None::new(Position::fake()),
                        Position::fake()
                    )
                    .into()
                ],
                Position::fake()
            ),
        ));
    }

    #[test]
    fn transform_map_with_spread_map() {
        insta::assert_debug_snapshot!(transform(
            &Context::dummy(Default::default(), Default::default()),
            &Map::new(
                types::None::new(Position::fake()),
                types::None::new(Position::fake()),
                vec![MapElement::Multiple(None::new(Position::fake()).into())],
                Position::fake()
            ),
        ));
    }
}
