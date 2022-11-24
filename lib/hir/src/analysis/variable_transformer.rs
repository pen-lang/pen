use crate::ir::*;

pub fn transform(module: &Module, transform: &dyn Fn(&Variable) -> Expression) -> Module {
    Module::new(
        module.type_definitions().to_vec(),
        module.type_aliases().to_vec(),
        module.foreign_declarations().to_vec(),
        module.function_declarations().to_vec(),
        module
            .function_definitions()
            .iter()
            .map(|definition| transform_function_definition(definition, transform))
            .collect(),
        module.position().clone(),
    )
}

fn transform_function_definition(
    definition: &FunctionDefinition,
    transform: &dyn Fn(&Variable) -> Expression,
) -> FunctionDefinition {
    FunctionDefinition::new(
        definition.name(),
        definition.original_name(),
        transform_lambda(definition.lambda(), transform),
        definition.foreign_definition_configuration().cloned(),
        definition.is_public(),
        definition.position().clone(),
    )
}

fn transform_lambda(lambda: &Lambda, transform: &dyn Fn(&Variable) -> Expression) -> Lambda {
    Lambda::new(
        lambda.arguments().to_vec(),
        lambda.result_type().clone(),
        transform_expression(lambda.body(), &|variable| {
            if lambda
                .arguments()
                .iter()
                .any(|argument| argument.name() == variable.name())
            {
                variable.clone().into()
            } else {
                transform(variable)
            }
        }),
        lambda.position().clone(),
    )
}

fn transform_expression(
    expression: &Expression,
    transform: &dyn Fn(&Variable) -> Expression,
) -> Expression {
    match expression {
        Expression::Call(call) => Call::new(
            call.function_type().cloned(),
            transform_expression(call.function(), transform),
            call.arguments()
                .iter()
                .map(|argument| transform_expression(argument, transform))
                .collect(),
            call.position().clone(),
        )
        .into(),
        Expression::If(if_) => If::new(
            transform_expression(if_.condition(), transform),
            transform_expression(if_.then(), transform),
            transform_expression(if_.else_(), transform),
            if_.position().clone(),
        )
        .into(),
        Expression::IfList(if_) => IfList::new(
            if_.type_().cloned(),
            transform_expression(if_.list(), transform),
            if_.first_name(),
            if_.rest_name(),
            transform_expression(if_.then(), &|variable| {
                if if_.first_name() == variable.name() || if_.rest_name() == variable.name() {
                    variable.clone().into()
                } else {
                    transform(variable)
                }
            }),
            transform_expression(if_.else_(), transform),
            if_.position().clone(),
        )
        .into(),
        Expression::IfMap(if_) => IfMap::new(
            if_.key_type().cloned(),
            if_.value_type().cloned(),
            if_.name(),
            transform_expression(if_.map(), transform),
            transform_expression(if_.key(), transform),
            transform_expression(if_.then(), &|variable| {
                if if_.name() == variable.name() {
                    variable.clone().into()
                } else {
                    transform(variable)
                }
            }),
            transform_expression(if_.else_(), transform),
            if_.position().clone(),
        )
        .into(),
        Expression::IfType(if_) => {
            let branch_transform = &|variable: &Variable| {
                if if_.name() == variable.name() {
                    variable.clone().into()
                } else {
                    transform(variable)
                }
            };

            IfType::new(
                if_.name(),
                transform_expression(if_.argument(), transform),
                if_.branches()
                    .iter()
                    .map(|branch| {
                        IfTypeBranch::new(
                            branch.type_().clone(),
                            transform_expression(branch.expression(), &branch_transform),
                        )
                    })
                    .collect(),
                if_.else_().map(|branch| {
                    ElseBranch::new(
                        branch.type_().cloned(),
                        transform_expression(branch.expression(), &branch_transform),
                        branch.position().clone(),
                    )
                }),
                if_.position().clone(),
            )
            .into()
        }
        Expression::Lambda(lambda) => transform_lambda(lambda, transform).into(),
        Expression::Let(let_) => Let::new(
            let_.name().map(String::from),
            let_.type_().cloned(),
            transform_expression(let_.bound_expression(), transform),
            transform_expression(let_.expression(), &|variable| {
                if let_.name() == Some(variable.name()) {
                    variable.clone().into()
                } else {
                    transform(variable)
                }
            }),
            let_.position().clone(),
        )
        .into(),
        Expression::List(list) => List::new(
            list.type_().clone(),
            list.elements()
                .iter()
                .map(|element| match element {
                    ListElement::Multiple(element) => {
                        ListElement::Multiple(transform_expression(element, transform))
                    }
                    ListElement::Single(element) => {
                        ListElement::Single(transform_expression(element, transform))
                    }
                })
                .collect(),
            list.position().clone(),
        )
        .into(),
        Expression::ListComprehension(comprehension) => {
            let mut branches = vec![];
            let mut transform: Box<dyn Fn(&Variable) -> Expression> = Box::new(transform);

            for branch in comprehension.branches() {
                let iteratee = transform_expression(branch.iteratee(), &transform);

                transform = Box::new(move |variable| {
                    if branch.primary_name() == variable.name()
                        || branch.secondary_name() == Some(variable.name())
                    {
                        variable.clone().into()
                    } else {
                        transform(variable)
                    }
                });

                branches.push(ListComprehensionBranch::new(
                    branch.type_().cloned(),
                    branch.primary_name(),
                    branch.secondary_name().map(String::from),
                    iteratee,
                    branch
                        .condition()
                        .map(|expression| transform_expression(expression, &transform)),
                    branch.position().clone(),
                ));
            }

            ListComprehension::new(
                comprehension.type_().clone(),
                transform_expression(comprehension.element(), &transform),
                branches,
                comprehension.position().clone(),
            )
            .into()
        }
        Expression::Map(map) => Map::new(
            map.key_type().clone(),
            map.value_type().clone(),
            map.elements()
                .iter()
                .map(|element| match element {
                    MapElement::Insertion(entry) => MapElement::Insertion(MapEntry::new(
                        transform_expression(entry.key(), transform),
                        transform_expression(entry.value(), transform),
                        entry.position().clone(),
                    )),
                    MapElement::Map(map) => MapElement::Map(transform_expression(map, transform)),
                })
                .collect(),
            map.position().clone(),
        )
        .into(),
        Expression::Operation(operation) => transform_operation(operation, transform).into(),
        Expression::RecordConstruction(construction) => RecordConstruction::new(
            construction.type_().clone(),
            construction
                .fields()
                .iter()
                .map(|field| {
                    RecordField::new(
                        field.name(),
                        transform_expression(field.expression(), transform),
                        field.position().clone(),
                    )
                })
                .collect(),
            construction.position().clone(),
        )
        .into(),
        Expression::RecordDeconstruction(deconstruction) => RecordDeconstruction::new(
            deconstruction.type_().cloned(),
            transform_expression(deconstruction.record(), transform),
            deconstruction.field_name(),
            deconstruction.position().clone(),
        )
        .into(),
        Expression::RecordUpdate(update) => RecordUpdate::new(
            update.type_().clone(),
            transform_expression(update.record(), transform),
            update
                .fields()
                .iter()
                .map(|field| {
                    RecordField::new(
                        field.name(),
                        transform_expression(field.expression(), transform),
                        field.position().clone(),
                    )
                })
                .collect(),
            update.position().clone(),
        )
        .into(),
        Expression::Thunk(thunk) => Thunk::new(
            thunk.type_().cloned(),
            transform_expression(thunk.expression(), transform),
            thunk.position().clone(),
        )
        .into(),
        Expression::TypeCoercion(coercion) => TypeCoercion::new(
            coercion.from().clone(),
            coercion.to().clone(),
            transform_expression(coercion.argument(), transform),
            coercion.position().clone(),
        )
        .into(),
        Expression::Variable(variable) => transform(variable),
        Expression::Boolean(_)
        | Expression::BuiltInFunction(_)
        | Expression::String(_)
        | Expression::None(_)
        | Expression::Number(_) => expression.clone(),
    }
}

fn transform_operation(
    operation: &Operation,
    transform: &dyn Fn(&Variable) -> Expression,
) -> Operation {
    match operation {
        Operation::Addition(operation) => AdditionOperation::new(
            operation.type_().cloned(),
            transform_expression(operation.lhs(), transform),
            transform_expression(operation.rhs(), transform),
            operation.position().clone(),
        )
        .into(),
        Operation::Arithmetic(operation) => ArithmeticOperation::new(
            operation.operator(),
            transform_expression(operation.lhs(), transform),
            transform_expression(operation.rhs(), transform),
            operation.position().clone(),
        )
        .into(),
        Operation::Boolean(operation) => BooleanOperation::new(
            operation.operator(),
            transform_expression(operation.lhs(), transform),
            transform_expression(operation.rhs(), transform),
            operation.position().clone(),
        )
        .into(),
        Operation::Equality(operation) => EqualityOperation::new(
            operation.type_().cloned(),
            operation.operator(),
            transform_expression(operation.lhs(), transform),
            transform_expression(operation.rhs(), transform),
            operation.position().clone(),
        )
        .into(),
        Operation::Not(operation) => NotOperation::new(
            transform_expression(operation.expression(), transform),
            operation.position().clone(),
        )
        .into(),
        Operation::Order(operation) => OrderOperation::new(
            operation.operator(),
            transform_expression(operation.lhs(), transform),
            transform_expression(operation.rhs(), transform),
            operation.position().clone(),
        )
        .into(),
        Operation::Try(operation) => TryOperation::new(
            operation.type_().cloned(),
            transform_expression(operation.expression(), transform),
            operation.position().clone(),
        )
        .into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test::{FunctionDefinitionFake, ModuleFake},
        types,
    };
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    #[test]
    fn transform_variable() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        Variable::new("x", Position::fake()),
                        Position::fake()
                    ),
                    false
                )],),
                &|variable| if variable.name() == "x" {
                    Variable::new("y", variable.position().clone()).into()
                } else {
                    variable.clone().into()
                }
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    Variable::new("y", Position::fake()),
                    Position::fake()
                ),
                false
            )])
        );
    }

    #[test]
    fn do_not_transform_variable_shadowed_by_argument() {
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
            "x",
            Lambda::new(
                vec![Argument::new("x", types::None::new(Position::fake()))],
                types::None::new(Position::fake()),
                Variable::new("x", Position::fake()),
                Position::fake(),
            ),
            false,
        )]);

        assert_eq!(
            transform(&module, &|variable| if variable.name() == "x" {
                Variable::new("y", variable.position().clone()).into()
            } else {
                variable.clone().into()
            }),
            module
        );
    }

    #[test]
    fn do_not_transform_variable_shadowed_by_statement() {
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
            "x",
            Lambda::new(
                vec![],
                types::None::new(Position::fake()),
                Let::new(
                    Some("x".into()),
                    None,
                    None::new(Position::fake()),
                    Variable::new("x", Position::fake()),
                    Position::fake(),
                ),
                Position::fake(),
            ),
            false,
        )]);

        assert_eq!(
            transform(&module, &|variable| if variable.name() == "x" {
                Variable::new("y", variable.position().clone()).into()
            } else {
                variable.clone().into()
            }),
            module
        );
    }

    #[test]
    fn do_not_transform_shadowed_variable_in_let() {
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
            "x",
            Lambda::new(
                vec![],
                types::None::new(Position::fake()),
                Let::new(
                    Some("x".into()),
                    None,
                    None::new(Position::fake()),
                    Variable::new("x", Position::fake()),
                    Position::fake(),
                ),
                Position::fake(),
            ),
            false,
        )]);

        assert_eq!(
            transform(&module, &|variable| if variable.name() == "x" {
                Variable::new("y", variable.position().clone()).into()
            } else {
                variable.clone().into()
            }),
            module
        );
    }

    mod list_comprehension {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn transform_shadowed_variable_in_iteratee() {
            let module = |name| {
                Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        ListComprehension::new(
                            types::Number::new(Position::fake()),
                            Variable::new("x", Position::fake()),
                            vec![ListComprehensionBranch::new(
                                None,
                                vec!["x".into()],
                                vec![Variable::new(name, Position::fake()).into()],
                                None,
                                Position::fake(),
                            )],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )])
            };

            assert_eq!(
                transform(&module("x"), &|variable| if variable.name() == "x" {
                    Variable::new("y", variable.position().clone()).into()
                } else {
                    variable.clone().into()
                }),
                module("y")
            );
        }

        #[test]
        fn transform_variable_in_condition() {
            let module = |name| {
                Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        ListComprehension::new(
                            types::Number::new(Position::fake()),
                            Variable::new("x", Position::fake()),
                            vec![ListComprehensionBranch::new(
                                None,
                                vec!["x".into()],
                                vec![Variable::new("xs", Position::fake()).into()],
                                Some(Variable::new(name, Position::fake()).into()),
                                Position::fake(),
                            )],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )])
            };

            assert_eq!(
                transform(&module("a"), &|variable| if variable.name() == "a" {
                    Variable::new("b", variable.position().clone()).into()
                } else {
                    variable.clone().into()
                }),
                module("b")
            );
        }

        #[test]
        fn transform_shadowed_variable_in_condition() {
            let module = Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    ListComprehension::new(
                        types::Number::new(Position::fake()),
                        Variable::new("x", Position::fake()),
                        vec![ListComprehensionBranch::new(
                            None,
                            vec!["x".into()],
                            vec![Variable::new("xs", Position::fake()).into()],
                            Some(Variable::new("x", Position::fake()).into()),
                            Position::fake(),
                        )],
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )]);

            assert_eq!(
                transform(&module, &|variable| if variable.name() == "x" {
                    Variable::new("y", variable.position().clone()).into()
                } else {
                    variable.clone().into()
                }),
                module
            );
        }

        #[test]
        fn transform_shadowed_primary_name() {
            let module = Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    ListComprehension::new(
                        types::None::new(Position::fake()),
                        Variable::new("x", Position::fake()),
                        vec![ListComprehensionBranch::new(
                            None,
                            vec!["x".into()],
                            vec![Variable::new("xs", Position::fake()).into()],
                            None,
                            Position::fake(),
                        )],
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )]);

            assert_eq!(
                transform(&module, &|variable| if variable.name() == "x" {
                    Variable::new("y", variable.position().clone()).into()
                } else {
                    variable.clone().into()
                }),
                module
            );
        }

        #[test]
        fn transform_shadowed_secondary_name() {
            let module = Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    ListComprehension::new(
                        types::Number::new(Position::fake()),
                        Variable::new("v", Position::fake()),
                        vec![ListComprehensionBranch::new(
                            None,
                            vec!["k".into(), "v".into()],
                            vec![Variable::new("xs", Position::fake()).into()],
                            None,
                            Position::fake(),
                        )],
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )]);

            assert_eq!(
                transform(&module, &|variable| if variable.name() == "v" {
                    Variable::new("x", variable.position().clone()).into()
                } else {
                    variable.clone().into()
                }),
                module
            );
        }

        #[test]
        fn transform_variable_shadowed_by_second_branch() {
            let module = Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    ListComprehension::new(
                        types::Number::new(Position::fake()),
                        Variable::new("y", Position::fake()),
                        vec![
                            ListComprehensionBranch::new(
                                None,
                                vec!["x".into()],
                                vec![Variable::new("xs", Position::fake()).into()],
                                None,
                                Position::fake(),
                            ),
                            ListComprehensionBranch::new(
                                None,
                                vec!["y".into()],
                                vec![Variable::new("x", Position::fake()).into()],
                                None,
                                Position::fake(),
                            ),
                        ],
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )]);

            assert_eq!(
                transform(&module, &|variable| if variable.name() == "y" {
                    Variable::new("z", variable.position().clone()).into()
                } else {
                    variable.clone().into()
                }),
                module
            );
        }
    }
}
