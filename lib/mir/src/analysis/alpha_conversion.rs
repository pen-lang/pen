mod context;

use self::context::Context;
use crate::ir::*;
use fnv::FnvHashMap;

pub fn transform(module: &Module) -> Module {
    let name_counts = module
        .foreign_declarations()
        .iter()
        .map(|declaration| declaration.name())
        .chain(
            module
                .function_declarations()
                .iter()
                .map(|declaration| declaration.name()),
        )
        .chain(
            module
                .function_definitions()
                .iter()
                .map(|definition| definition.definition().name()),
        )
        .map(|name| (name, 1))
        .collect::<FnvHashMap<_, _>>();

    Module::new(
        module.type_definitions().to_vec(),
        module.foreign_declarations().to_vec(),
        module.foreign_definitions().to_vec(),
        module.function_declarations().to_vec(),
        module
            .function_definitions()
            .iter()
            .map(|definition| {
                GlobalFunctionDefinition::new(
                    {
                        let definition = definition.definition();

                        FunctionDefinition::with_options(
                            definition.name(),
                            definition.environment().to_vec(),
                            definition.arguments().to_vec(),
                            definition.result_type().clone(),
                            transform_expression(
                                &Context::new(name_counts.clone()),
                                definition.body(),
                                &Default::default(),
                            ),
                            definition.is_thunk(),
                        )
                    },
                    definition.is_public(),
                )
            })
            .collect(),
        module.type_information().clone(),
    )
}

fn transform_expression<'a>(
    context: &Context<'a>,
    expression: &'a Expression,
    variables: &hamt::Map<&'a str, String>,
) -> Expression {
    let transform = |expression| transform_expression(context, expression, variables);

    match expression {
        Expression::ArithmeticOperation(operation) => ArithmeticOperation::new(
            operation.operator(),
            transform(operation.lhs()),
            transform(operation.rhs()),
        )
        .into(),
        Expression::Case(case) => Case::new(
            transform(case.argument()),
            case.alternatives()
                .iter()
                .map(|alternative| {
                    let name = context.rename(alternative.name());

                    Alternative::new(
                        alternative.types().to_vec(),
                        &name,
                        transform_expression(
                            context,
                            alternative.expression(),
                            &variables.insert(alternative.name(), name.clone()),
                        ),
                    )
                })
                .collect(),
            case.default_alternative().map(|alternative| {
                let name = context.rename(alternative.name());

                DefaultAlternative::new(
                    &name,
                    transform_expression(
                        context,
                        alternative.expression(),
                        &variables.insert(alternative.name(), name.clone()),
                    ),
                )
            }),
        )
        .into(),
        Expression::CloneVariables(clone) => {
            CloneVariables::new(clone.variables().clone(), transform(clone.expression())).into()
        }
        Expression::ComparisonOperation(operation) => ComparisonOperation::new(
            operation.operator(),
            transform(operation.lhs()),
            transform(operation.rhs()),
        )
        .into(),
        Expression::DropVariables(drop) => {
            DropVariables::new(drop.variables().clone(), transform(drop.expression())).into()
        }
        Expression::Call(call) => Call::new(
            call.type_().clone(),
            transform(call.function()),
            call.arguments().iter().map(transform).collect(),
        )
        .into(),
        Expression::If(if_) => If::new(
            transform(if_.condition()),
            transform(if_.then()),
            transform(if_.else_()),
        )
        .into(),
        Expression::Let(let_) => {
            let name = context.rename(let_.name());

            Let::new(
                &name,
                let_.type_().clone(),
                transform(let_.bound_expression()),
                transform_expression(
                    context,
                    let_.expression(),
                    &variables.insert(let_.name(), name.clone()),
                ),
            )
            .into()
        }
        Expression::LetRecursive(let_) => {
            let definition = let_.definition();
            let name = context.rename(definition.name());
            let variables = variables.insert(definition.name(), name.clone());

            LetRecursive::new(
                {
                    let variables = variables.extend(
                        definition
                            .arguments()
                            .iter()
                            .map(|argument| (argument.name(), context.rename(argument.name()))),
                    );

                    FunctionDefinition::with_options(
                        name,
                        definition
                            .environment()
                            .iter()
                            .map(|free_variable| {
                                Argument::new(
                                    variables
                                        .get(free_variable.name())
                                        .map(String::as_str)
                                        .unwrap_or_else(|| free_variable.name()),
                                    free_variable.type_().clone(),
                                )
                            })
                            .collect(),
                        definition
                            .arguments()
                            .iter()
                            .map(|argument| {
                                Argument::new(
                                    variables
                                        .get(argument.name())
                                        .map(String::as_str)
                                        .unwrap_or_else(|| argument.name()),
                                    argument.type_().clone(),
                                )
                            })
                            .collect(),
                        definition.result_type().clone(),
                        transform_expression(context, definition.body(), &variables),
                        definition.is_thunk(),
                    )
                },
                transform_expression(context, let_.expression(), &variables),
            )
            .into()
        }
        Expression::Synchronize(synchronize) => Synchronize::new(
            synchronize.type_().clone(),
            transform(synchronize.expression()),
        )
        .into(),
        Expression::Record(record) => Record::new(
            record.type_().clone(),
            record.fields().iter().map(transform).collect(),
        )
        .into(),
        Expression::RecordField(field) => RecordField::new(
            field.type_().clone(),
            field.index(),
            transform(field.record()),
        )
        .into(),
        Expression::RecordUpdate(update) => RecordUpdate::new(
            update.type_().clone(),
            transform(update.record()),
            update
                .fields()
                .iter()
                .map(|field| RecordUpdateField::new(field.index(), transform(field.expression())))
                .collect(),
        )
        .into(),
        Expression::StringConcatenation(concatenation) => {
            StringConcatenation::new(concatenation.operands().iter().map(transform).collect())
                .into()
        }
        Expression::TryOperation(operation) => {
            let name = context.rename(operation.name());

            TryOperation::new(
                transform(operation.operand()),
                &name,
                operation.type_().clone(),
                transform_expression(
                    context,
                    operation.then(),
                    &variables.insert(operation.name(), name.clone()),
                ),
            )
            .into()
        }
        Expression::TypeInformationFunction(information) => {
            TypeInformationFunction::new(transform(information.variant())).into()
        }
        Expression::Variant(variant) => {
            Variant::new(variant.type_().clone(), transform(variant.payload())).into()
        }
        Expression::Variable(variable) => if let Some(name) = variables.get(variable.name()) {
            Variable::new(name)
        } else {
            variable.clone()
        }
        .into(),
        Expression::Boolean(_)
        | Expression::ByteString(_)
        | Expression::None
        | Expression::Number(_) => expression.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test::ModuleFake, types::Type};
    use pretty_assertions::assert_eq;

    #[test]
    fn transform_empty_module() {
        assert_eq!(transform(&Module::empty()), Module::empty());
    }

    #[test]
    fn transform_number() {
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::new(
            "f",
            vec![],
            Type::Number,
            42.0,
        )]);

        assert_eq!(transform(&module), module);
    }

    #[test]
    fn transform_case() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    Let::new(
                        "x",
                        Type::Number,
                        42.0,
                        Case::new(
                            Variable::new("x"),
                            vec![Alternative::new(
                                vec![Type::Number],
                                "x",
                                Variable::new("x"),
                            )],
                            Some(DefaultAlternative::new("x", Variable::new("x"))),
                        ),
                    )
                )])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![],
                Type::Number,
                Let::new(
                    "x",
                    Type::Number,
                    42.0,
                    Case::new(
                        Variable::new("x"),
                        vec![Alternative::new(
                            vec![Type::Number],
                            "x:1",
                            Variable::new("x:1"),
                        )],
                        Some(DefaultAlternative::new("x:2", Variable::new("x:2"))),
                    ),
                )
            )])
        );
    }

    #[test]
    fn transform_let() {
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::new(
            "f",
            vec![],
            Type::Number,
            Let::new("x", Type::Number, 42.0, Variable::new("x")),
        )]);

        assert_eq!(transform(&module), module);
    }

    #[test]
    fn transform_let_with_shadowed_variable() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    Let::new(
                        "x",
                        Type::Number,
                        42.0,
                        Let::new("x", Type::Number, Variable::new("x"), Variable::new("x")),
                    )
                )])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![],
                Type::Number,
                Let::new(
                    "x",
                    Type::Number,
                    42.0,
                    Let::new(
                        "x:1",
                        Type::Number,
                        Variable::new("x"),
                        Variable::new("x:1")
                    ),
                )
            )])
        );
    }

    #[test]
    fn transform_let_recursive() {
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::new(
            "f",
            vec![],
            Type::Number,
            LetRecursive::new(
                FunctionDefinition::new("g", vec![], Type::Number, Variable::new("g")),
                Variable::new("g"),
            ),
        )]);

        assert_eq!(transform(&module), module);
    }

    #[test]
    fn transform_let_recursive_with_shadowed_function_name() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    Let::new(
                        "g",
                        Type::Number,
                        42.0,
                        LetRecursive::new(
                            FunctionDefinition::new("g", vec![], Type::Number, Variable::new("g")),
                            Variable::new("g"),
                        )
                    )
                )])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![],
                Type::Number,
                Let::new(
                    "g",
                    Type::Number,
                    42.0,
                    LetRecursive::new(
                        FunctionDefinition::new("g:1", vec![], Type::Number, Variable::new("g:1")),
                        Variable::new("g:1"),
                    )
                )
            )])
        );
    }

    #[test]
    fn transform_let_recursive_with_shadowed_argument_name() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    Let::new(
                        "g",
                        Type::Number,
                        42.0,
                        LetRecursive::new(
                            FunctionDefinition::new(
                                "h",
                                vec![Argument::new("g", Type::Number)],
                                Type::Number,
                                Variable::new("g")
                            ),
                            Variable::new("g"),
                        )
                    )
                )])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![],
                Type::Number,
                Let::new(
                    "g",
                    Type::Number,
                    42.0,
                    LetRecursive::new(
                        FunctionDefinition::new(
                            "h",
                            vec![Argument::new("g:1", Type::Number)],
                            Type::Number,
                            Variable::new("g:1")
                        ),
                        Variable::new("g"),
                    )
                )
            )])
        );
    }

    #[test]
    fn transform_let_recursive_with_free_variable() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    Let::new(
                        "x",
                        Type::Number,
                        42.0,
                        Let::new(
                            "x",
                            Type::Number,
                            42.0,
                            LetRecursive::new(
                                FunctionDefinition::with_options(
                                    "h",
                                    vec![Argument::new("x", Type::Number)],
                                    vec![],
                                    Type::Number,
                                    Variable::new("x"),
                                    false,
                                ),
                                Variable::new("x"),
                            )
                        )
                    )
                )])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![],
                Type::Number,
                Let::new(
                    "x",
                    Type::Number,
                    42.0,
                    Let::new(
                        "x:1",
                        Type::Number,
                        42.0,
                        LetRecursive::new(
                            FunctionDefinition::with_options(
                                "h",
                                vec![Argument::new("x:1", Type::Number)],
                                vec![],
                                Type::Number,
                                Variable::new("x:1"),
                                false,
                            ),
                            Variable::new("x:1"),
                        )
                    )
                )
            )])
        );
    }

    #[test]
    fn transform_let_recursive_with_shadowed_global_function_name() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    LetRecursive::new(
                        FunctionDefinition::with_options(
                            "f",
                            vec![],
                            vec![],
                            Type::Number,
                            Expression::None,
                            false,
                        ),
                        Expression::None,
                    )
                )])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![],
                Type::Number,
                LetRecursive::new(
                    FunctionDefinition::with_options(
                        "f:1",
                        vec![],
                        vec![],
                        Type::Number,
                        Expression::None,
                        false,
                    ),
                    Expression::None,
                )
            )])
        );
    }

    #[test]
    fn transform_let_recursive_with_shadowed_nested_function_name() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    LetRecursive::new(
                        FunctionDefinition::with_options(
                            "f",
                            vec![],
                            vec![],
                            Type::Number,
                            LetRecursive::new(
                                FunctionDefinition::with_options(
                                    "f",
                                    vec![],
                                    vec![],
                                    Type::Number,
                                    Expression::None,
                                    false,
                                ),
                                Expression::None,
                            ),
                            false,
                        ),
                        Expression::None,
                    )
                )])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![],
                Type::Number,
                LetRecursive::new(
                    FunctionDefinition::with_options(
                        "f:1",
                        vec![],
                        vec![],
                        Type::Number,
                        LetRecursive::new(
                            FunctionDefinition::with_options(
                                "f:2",
                                vec![],
                                vec![],
                                Type::Number,
                                Expression::None,
                                false,
                            ),
                            Expression::None,
                        ),
                        false,
                    ),
                    Expression::None,
                )
            )])
        );
    }

    #[test]
    fn transform_try_operation() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::Number,
                    Let::new(
                        "x",
                        Type::Number,
                        1.0,
                        TryOperation::new(2.0, "x", Type::Number, Variable::new("x")),
                    )
                )])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![],
                Type::Number,
                Let::new(
                    "x",
                    Type::Number,
                    1.0,
                    TryOperation::new(2.0, "x:1", Type::Number, Variable::new("x:1")),
                )
            )])
        );
    }
}
