use crate::ListTypeConfiguration;
use hir::{
    ir::*,
    types::{self, Type},
};
use position::Position;

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
        [ListElement::Multiple(expression)] => Call::new(
            Some(
                types::Function::new(
                    vec![
                        types::Function::new(vec![], any_list_type.clone(), position.clone())
                            .into(),
                    ],
                    any_list_type.clone(),
                    position.clone(),
                )
                .into(),
            ),
            Variable::new(&configuration.lazy_function_name, position.clone()),
            vec![Thunk::new(
                Some(any_list_type.into()),
                expression.clone(),
                position.clone(),
            )
            .into()],
            position.clone(),
        )
        .into(),
        [ListElement::Multiple(expression), ..] => Call::new(
            Some(
                types::Function::new(
                    vec![
                        types::Function::new(vec![], any_list_type.clone(), position.clone())
                            .into(),
                        any_list_type.clone().into(),
                    ],
                    any_list_type.clone(),
                    position.clone(),
                )
                .into(),
            ),
            Variable::new(&configuration.concatenate_function_name, position.clone()),
            vec![
                Thunk::new(
                    Some(any_list_type.into()),
                    expression.clone(),
                    position.clone(),
                )
                .into(),
                rest_expression(),
            ],
            position.clone(),
        )
        .into(),
        [ListElement::Single(expression), ..] => Call::new(
            Some(
                types::Function::new(
                    vec![
                        types::Function::new(
                            vec![],
                            types::Any::new(position.clone()),
                            position.clone(),
                        )
                        .into(),
                        any_list_type.clone().into(),
                    ],
                    any_list_type,
                    position.clone(),
                )
                .into(),
            ),
            Variable::new(&configuration.prepend_function_name, position.clone()),
            vec![
                Thunk::new(
                    Some(types::Any::new(position.clone()).into()),
                    TypeCoercion::new(
                        type_.clone(),
                        types::Any::new(position.clone()),
                        expression.clone(),
                        position.clone(),
                    ),
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
    use crate::list_type_configuration::LIST_TYPE_CONFIGURATION;
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    fn get_list_type() -> types::Reference {
        types::Reference::new(
            LIST_TYPE_CONFIGURATION.list_type_name.clone(),
            Position::fake(),
        )
    }

    fn get_prepend_function_type() -> types::Function {
        let list_type = get_list_type();

        types::Function::new(
            vec![
                types::Function::new(vec![], types::Any::new(Position::fake()), Position::fake())
                    .into(),
                list_type.clone().into(),
            ],
            list_type,
            Position::fake(),
        )
    }

    fn get_concatenate_function_type() -> types::Function {
        let list_type = get_list_type();

        types::Function::new(
            vec![
                types::Function::new(vec![], list_type.clone(), Position::fake()).into(),
                list_type.clone().into(),
            ],
            list_type,
            Position::fake(),
        )
    }

    fn get_lazy_function_type() -> types::Function {
        let list_type = get_list_type();

        types::Function::new(
            vec![types::Function::new(vec![], list_type.clone(), Position::fake()).into()],
            list_type,
            Position::fake(),
        )
    }

    #[test]
    fn transform_empty_list() {
        let list_type = get_list_type();

        assert_eq!(
            transform(
                &List::new(types::None::new(Position::fake()), vec![], Position::fake()),
                &LIST_TYPE_CONFIGURATION,
            ),
            Call::new(
                Some(types::Function::new(vec![], list_type, Position::fake()).into()),
                Variable::new(
                    &LIST_TYPE_CONFIGURATION.empty_list_function_name,
                    Position::fake()
                ),
                vec![],
                Position::fake()
            )
            .into()
        );
    }

    #[test]
    fn transform_list_with_one_element() {
        let list_type = get_list_type();

        assert_eq!(
            transform(
                &List::new(
                    types::None::new(Position::fake()),
                    vec![ListElement::Single(None::new(Position::fake()).into())],
                    Position::fake()
                ),
                &LIST_TYPE_CONFIGURATION,
            ),
            Call::new(
                Some(get_prepend_function_type().into()),
                Variable::new(
                    &LIST_TYPE_CONFIGURATION.prepend_function_name,
                    Position::fake()
                ),
                vec![
                    Thunk::new(
                        Some(types::Any::new(Position::fake()).into()),
                        TypeCoercion::new(
                            types::None::new(Position::fake()),
                            types::Any::new(Position::fake()),
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    )
                    .into(),
                    Call::new(
                        Some(types::Function::new(vec![], list_type, Position::fake()).into()),
                        Variable::new(
                            &LIST_TYPE_CONFIGURATION.empty_list_function_name,
                            Position::fake()
                        ),
                        vec![],
                        Position::fake()
                    )
                    .into(),
                ],
                Position::fake(),
            )
            .into(),
        );
    }

    #[test]
    fn transform_list_with_two_elements() {
        let list_type = get_list_type();
        let prepend_function_type = get_prepend_function_type();

        assert_eq!(
            transform(
                &List::new(
                    types::None::new(Position::fake()),
                    vec![
                        ListElement::Single(None::new(Position::fake()).into()),
                        ListElement::Single(None::new(Position::fake()).into())
                    ],
                    Position::fake()
                ),
                &LIST_TYPE_CONFIGURATION,
            ),
            Call::new(
                Some(prepend_function_type.clone().into()),
                Variable::new(
                    &LIST_TYPE_CONFIGURATION.prepend_function_name,
                    Position::fake()
                ),
                vec![
                    Thunk::new(
                        Some(types::Any::new(Position::fake()).into()),
                        TypeCoercion::new(
                            types::None::new(Position::fake()),
                            types::Any::new(Position::fake()),
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake()
                    )
                    .into(),
                    Call::new(
                        Some(prepend_function_type.into(),),
                        Variable::new(
                            &LIST_TYPE_CONFIGURATION.prepend_function_name,
                            Position::fake()
                        ),
                        vec![
                            Thunk::new(
                                Some(types::Any::new(Position::fake()).into()),
                                TypeCoercion::new(
                                    types::None::new(Position::fake()),
                                    types::Any::new(Position::fake()),
                                    None::new(Position::fake()),
                                    Position::fake(),
                                ),
                                Position::fake()
                            )
                            .into(),
                            Call::new(
                                Some(
                                    types::Function::new(vec![], list_type, Position::fake())
                                        .into()
                                ),
                                Variable::new(
                                    &LIST_TYPE_CONFIGURATION.empty_list_function_name,
                                    Position::fake()
                                ),
                                vec![],
                                Position::fake()
                            )
                            .into(),
                        ],
                        Position::fake(),
                    )
                    .into(),
                ],
                Position::fake(),
            )
            .into(),
        );
    }

    #[test]
    fn transform_multiple_elements() {
        let list_type = get_list_type();

        assert_eq!(
            transform(
                &List::new(
                    types::None::new(Position::fake()),
                    vec![ListElement::Multiple(
                        Variable::new("xs", Position::fake()).into()
                    )],
                    Position::fake()
                ),
                &LIST_TYPE_CONFIGURATION,
            ),
            Call::new(
                Some(get_lazy_function_type().into()),
                Variable::new(
                    &LIST_TYPE_CONFIGURATION.lazy_function_name,
                    Position::fake()
                ),
                vec![Thunk::new(
                    Some(list_type.into()),
                    Variable::new("xs", Position::fake()),
                    Position::fake(),
                )
                .into()],
                Position::fake(),
            )
            .into(),
        );
    }

    #[test]
    fn transform_two_multiple_elements() {
        let list_type = get_list_type();

        assert_eq!(
            transform(
                &List::new(
                    types::None::new(Position::fake()),
                    vec![
                        ListElement::Multiple(Variable::new("xs", Position::fake()).into()),
                        ListElement::Multiple(Variable::new("ys", Position::fake()).into())
                    ],
                    Position::fake()
                ),
                &LIST_TYPE_CONFIGURATION,
            ),
            Call::new(
                Some(get_concatenate_function_type().into()),
                Variable::new(
                    &LIST_TYPE_CONFIGURATION.concatenate_function_name,
                    Position::fake()
                ),
                vec![
                    Thunk::new(
                        Some(list_type.clone().into()),
                        Variable::new("xs", Position::fake()),
                        Position::fake(),
                    )
                    .into(),
                    Call::new(
                        Some(get_lazy_function_type().into()),
                        Variable::new(
                            &LIST_TYPE_CONFIGURATION.lazy_function_name,
                            Position::fake()
                        ),
                        vec![Thunk::new(
                            Some(list_type.into()),
                            Variable::new("ys", Position::fake()),
                            Position::fake(),
                        )
                        .into()],
                        Position::fake(),
                    )
                    .into(),
                ],
                Position::fake(),
            )
            .into(),
        );
    }
}
