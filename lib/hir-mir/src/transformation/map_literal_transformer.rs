use super::{equal_operation_transformer, hash_calculation_transformer};
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
            vec![
                equal_operation_transformer::transform_any_function(context, key_type, position)?
                    .into(),
                hash_calculation_transformer::transform_any_function(context, key_type, position)?
                    .into(),
                equal_operation_transformer::transform_any_function(context, value_type, position)?
                    .into(),
                hash_calculation_transformer::transform_any_function(
                    context, value_type, position,
                )?
                .into(),
            ],
            position.clone(),
        )
        .into(),
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
