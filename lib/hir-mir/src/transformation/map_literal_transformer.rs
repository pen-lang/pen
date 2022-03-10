use crate::MapTypeConfiguration;
use hir::{
    ir::*,
    types::{self, Type},
};
use position::Position;

pub fn transform(map: &Map, configuration: &MapTypeConfiguration) -> Expression {
    transform_map(
        map.key_type(),
        map.value_type(),
        map.elements(),
        map.position(),
        configuration,
    )
}

fn transform_map(
    key_type: &Type,
    value_type: &Type,
    elements: &[MapElement],
    position: &Position,
    configuration: &MapTypeConfiguration,
) -> Expression {
    let any_map_type = types::Reference::new(configuration.map_type_name.clone(), position.clone());

    match elements {
        [] => Call::new(
            Some(types::Function::new(vec![], any_map_type, position.clone()).into()),
            Variable::new(&configuration.empty_function_name, position.clone()),
            vec![todo!(), todo!(), todo!(), todo!()],
            position.clone(),
        )
        .into(),
        [element, ..] => {
            let rest_expression = transform_map(
                key_type,
                value_type,
                &elements[1..],
                position,
                configuration,
            );

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
    }
}
