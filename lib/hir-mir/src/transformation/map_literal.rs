use super::{collection_type, map_context};
use crate::{context::CompileContext, CompileError};
use hir::{
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
    let map_context_type = collection_type::transform_map_context(context, position)?;
    let any_map_type = collection_type::transform_map(context, position)?;
    let map_context = map_context::transform(context, key_type, value_type, position)?;

    Ok(match elements {
        [] => Call::new(
            Some(types::Function::new(vec![], any_map_type, position.clone()).into()),
            Variable::new(&configuration.empty_function_name, position.clone()),
            vec![],
            position.clone(),
        )
        .into(),
        // TODO Optimize cases where only a single element of MapElement::Map
        // exists when we pass context functions dynamically.
        [.., element] => {
            let rest_expression = transform_map(
                context,
                key_type,
                value_type,
                &elements[..elements.len() - 1],
                position,
            )?;

            match element {
                MapElement::Insertion(entry) => Call::new(
                    Some(
                        types::Function::new(
                            vec![
                                map_context_type,
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
                        map_context,
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
                            vec![map_context_type, any_map_type.clone(), any_map_type.clone()],
                            any_map_type,
                            position.clone(),
                        )
                        .into(),
                    ),
                    Variable::new(&configuration.merge_function_name, position.clone()),
                    vec![map_context, expression.clone(), rest_expression],
                    position.clone(),
                )
                .into(),
                MapElement::Removal(key) => Call::new(
                    Some(
                        types::Function::new(
                            vec![
                                map_context_type,
                                any_map_type.clone(),
                                types::Any::new(position.clone()).into(),
                            ],
                            any_map_type,
                            position.clone(),
                        )
                        .into(),
                    ),
                    Variable::new(&configuration.delete_function_name, position.clone()),
                    vec![
                        map_context,
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
