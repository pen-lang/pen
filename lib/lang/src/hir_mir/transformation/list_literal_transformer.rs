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
            Variable::new(&configuration.empty_list_function_name, position.clone()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hir_mir::dummy_type_configurations::DUMMY_LIST_TYPE_CONFIGURATION;
    use pretty_assertions::assert_eq;

    fn create_list_type() -> types::Reference {
        types::Reference::new(
            DUMMY_LIST_TYPE_CONFIGURATION.list_type_name.clone(),
            Position::dummy(),
        )
    }

    #[test]
    fn transform_empty_list() {
        let list_type = create_list_type();

        assert_eq!(
            transform(
                &List::new(
                    types::None::new(Position::dummy()),
                    vec![],
                    Position::dummy()
                ),
                &DUMMY_LIST_TYPE_CONFIGURATION,
            ),
            Call::new(
                Some(types::Function::new(vec![], list_type, Position::dummy()).into()),
                Variable::new(
                    &DUMMY_LIST_TYPE_CONFIGURATION.empty_list_function_name,
                    Position::dummy()
                ),
                vec![],
                Position::dummy()
            )
            .into()
        );
    }

    #[test]
    fn transform_list_with_one_element() {
        let list_type = create_list_type();

        assert_eq!(
            transform(
                &List::new(
                    types::None::new(Position::dummy()),
                    vec![ListElement::Single(None::new(Position::dummy()).into())],
                    Position::dummy()
                ),
                &DUMMY_LIST_TYPE_CONFIGURATION,
            ),
            Call::new(
                Some(
                    types::Function::new(
                        vec![
                            types::Any::new(Position::dummy()).into(),
                            list_type.clone().clone().into(),
                        ],
                        list_type.clone(),
                        Position::dummy(),
                    )
                    .into(),
                ),
                Variable::new(
                    &DUMMY_LIST_TYPE_CONFIGURATION.prepend_function_name,
                    Position::dummy()
                ),
                vec![
                    TypeCoercion::new(
                        types::None::new(Position::dummy()),
                        types::Any::new(Position::dummy()),
                        None::new(Position::dummy()),
                        Position::dummy(),
                    )
                    .into(),
                    Call::new(
                        Some(types::Function::new(vec![], list_type, Position::dummy()).into()),
                        Variable::new(
                            &DUMMY_LIST_TYPE_CONFIGURATION.empty_list_function_name,
                            Position::dummy()
                        ),
                        vec![],
                        Position::dummy()
                    )
                    .into(),
                ],
                Position::dummy(),
            )
            .into(),
        );
    }

    #[test]
    fn transform_list_with_two_elements() {
        let list_type = create_list_type();

        assert_eq!(
            transform(
                &List::new(
                    types::None::new(Position::dummy()),
                    vec![
                        ListElement::Single(None::new(Position::dummy()).into()),
                        ListElement::Single(None::new(Position::dummy()).into())
                    ],
                    Position::dummy()
                ),
                &DUMMY_LIST_TYPE_CONFIGURATION,
            ),
            Call::new(
                Some(
                    types::Function::new(
                        vec![
                            types::Any::new(Position::dummy()).into(),
                            list_type.clone().clone().into(),
                        ],
                        list_type.clone(),
                        Position::dummy(),
                    )
                    .into(),
                ),
                Variable::new(
                    &DUMMY_LIST_TYPE_CONFIGURATION.prepend_function_name,
                    Position::dummy()
                ),
                vec![
                    TypeCoercion::new(
                        types::None::new(Position::dummy()),
                        types::Any::new(Position::dummy()),
                        None::new(Position::dummy()),
                        Position::dummy(),
                    )
                    .into(),
                    Call::new(
                        Some(
                            types::Function::new(
                                vec![
                                    types::Any::new(Position::dummy()).into(),
                                    list_type.clone().clone().into(),
                                ],
                                list_type.clone(),
                                Position::dummy(),
                            )
                            .into(),
                        ),
                        Variable::new(
                            &DUMMY_LIST_TYPE_CONFIGURATION.prepend_function_name,
                            Position::dummy()
                        ),
                        vec![
                            TypeCoercion::new(
                                types::None::new(Position::dummy()),
                                types::Any::new(Position::dummy()),
                                None::new(Position::dummy()),
                                Position::dummy(),
                            )
                            .into(),
                            Call::new(
                                Some(
                                    types::Function::new(vec![], list_type, Position::dummy())
                                        .into()
                                ),
                                Variable::new(
                                    &DUMMY_LIST_TYPE_CONFIGURATION.empty_list_function_name,
                                    Position::dummy()
                                ),
                                vec![],
                                Position::dummy()
                            )
                            .into(),
                        ],
                        Position::dummy(),
                    )
                    .into(),
                ],
                Position::dummy(),
            )
            .into(),
        );
    }
}
