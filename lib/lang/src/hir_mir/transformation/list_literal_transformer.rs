use crate::{
    hir::*,
    hir_mir::ListTypeConfiguration,
    position::Position,
    types::{self, Type},
};

pub fn transform(list: &List, configuration: &ListTypeConfiguration) -> Expression {
    transform_list(
        list.type_(),
        list.elements(),
        list.position(),
        configuration,
    )
}

fn transform_list(
    type_: &Type,
    elements: &[ListElement],
    position: &Position,
    configuration: &ListTypeConfiguration,
) -> Expression {
    let rest_expression = || transform_list(type_, &elements[1..], position, configuration);
    let any_list_type =
        types::Reference::new(configuration.list_type_name.clone(), position.clone());

    match elements {
        [] => Call::new(
            Some(types::Function::new(vec![], any_list_type, position.clone()).into()),
            Variable::new(&configuration.empty_list_variable_name, position.clone()),
            vec![],
            position.clone(),
        )
        .into(),
        [ListElement::Multiple(expression), ..] => Call::new(
            Some(
                types::Function::new(
                    vec![any_list_type.clone().into(), any_list_type.clone().into()],
                    any_list_type,
                    position.clone(),
                )
                .into(),
            ),
            Variable::new(&configuration.concatenate_function_name, position.clone()),
            vec![expression.clone(), rest_expression()],
            position.clone(),
        )
        .into(),
        [ListElement::Single(expression), ..] => Call::new(
            Some(
                types::Function::new(
                    vec![
                        types::Any::new(position.clone()).into(),
                        any_list_type.clone().into(),
                    ],
                    any_list_type,
                    position.clone(),
                )
                .into(),
            ),
            Variable::new(&configuration.prepend_function_name, position.clone()),
            vec![
                TypeCoercion::new(
                    type_.clone(),
                    types::Any::new(position.clone()),
                    expression.clone(),
                    position.clone(),
                )
                .into(),
                rest_expression(),
            ],
            position.clone(),
        )
        .into(),
    }
}
