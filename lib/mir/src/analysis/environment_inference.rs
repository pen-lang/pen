use super::free_variable::find_free_variables;
use crate::{ir::*, types::Type};
use fnv::FnvHashMap;

pub fn transform(module: &Module) -> Module {
    Module::new(
        module.type_definitions().to_vec(),
        module.foreign_declarations().to_vec(),
        module.foreign_definitions().to_vec(),
        module.function_declarations().to_vec(),
        module
            .function_definitions()
            .iter()
            .map(transform_global_function_definition)
            .collect(),
        module.type_information().clone(),
    )
}

fn transform_global_function_definition(
    definition: &GlobalFunctionDefinition,
) -> GlobalFunctionDefinition {
    let public = definition.is_public();
    let definition = definition.definition();

    GlobalFunctionDefinition::new(
        FunctionDefinition::with_options(
            definition.name(),
            vec![],
            definition.arguments().to_vec(),
            definition.result_type().clone(),
            transform_expression(
                definition.body(),
                &definition
                    .arguments()
                    .iter()
                    .map(|argument| (argument.name().into(), argument.type_().clone()))
                    .collect(),
            ),
            definition.is_thunk(),
        ),
        public,
    )
}

fn transform_local_function_definition(
    definition: &FunctionDefinition,
    variables: &FnvHashMap<String, Type>,
) -> FunctionDefinition {
    let local_variables = [(definition.name().into(), definition.type_().clone().into())]
        .into_iter()
        .chain(
            definition
                .arguments()
                .iter()
                .map(|argument| (argument.name().into(), argument.type_().clone())),
        )
        .collect::<FnvHashMap<_, _>>();

    FunctionDefinition::with_options(
        definition.name(),
        find_free_variables(definition.body())
            .into_iter()
            .filter(|name| !local_variables.contains_key(name.as_str()))
            .filter_map(|name| {
                variables
                    .get(&name)
                    .map(|type_| Argument::new(name, type_.clone()))
            })
            .collect(),
        definition.arguments().to_vec(),
        definition.result_type().clone(),
        transform_expression(
            definition.body(),
            &variables
                .clone()
                .into_iter()
                .chain(local_variables)
                .collect(),
        ),
        definition.is_thunk(),
    )
}

fn transform_expression(
    expression: &Expression,
    variables: &FnvHashMap<String, Type>,
) -> Expression {
    let transform = |expression| transform_expression(expression, variables);

    match expression {
        Expression::ArithmeticOperation(operation) => ArithmeticOperation::new(
            operation.operator(),
            transform(operation.lhs()),
            transform(operation.rhs()),
        )
        .into(),
        Expression::Call(call) => Call::new(
            call.type_().clone(),
            transform(call.function()),
            call.arguments().iter().map(transform).collect(),
        )
        .into(),
        Expression::Case(case) => Case::new(
            transform(case.argument()),
            case.alternatives()
                .iter()
                .map(|alternative| {
                    let mut variables = variables.clone();

                    variables.insert(alternative.name().into(), alternative.type_().clone());

                    Alternative::new(
                        alternative.types().to_vec(),
                        alternative.name(),
                        transform_expression(alternative.expression(), &variables),
                    )
                })
                .collect(),
            case.default_alternative().map(|alternative| {
                let mut variables = variables.clone();

                variables.insert(alternative.name().into(), Type::Variant);

                DefaultAlternative::new(
                    alternative.name(),
                    transform_expression(alternative.expression(), &variables),
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
        Expression::If(if_) => If::new(
            transform(if_.condition()),
            transform(if_.then()),
            transform(if_.else_()),
        )
        .into(),
        Expression::Let(let_) => Let::new(
            let_.name(),
            let_.type_().clone(),
            transform(let_.bound_expression()),
            transform_expression(
                let_.expression(),
                &variables
                    .clone()
                    .into_iter()
                    .chain([(let_.name().into(), let_.type_().clone())])
                    .collect(),
            ),
        )
        .into(),
        Expression::LetRecursive(let_) => LetRecursive::new(
            transform_local_function_definition(let_.definition(), variables),
            transform_expression(
                let_.expression(),
                &variables
                    .clone()
                    .into_iter()
                    .chain([(
                        let_.definition().name().into(),
                        let_.definition().type_().clone().into(),
                    )])
                    .collect(),
            ),
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
        Expression::Synchronize(synchronize) => Synchronize::new(
            synchronize.type_().clone(),
            transform(synchronize.expression()),
        )
        .into(),
        Expression::TryOperation(operation) => TryOperation::new(
            transform(operation.operand()),
            operation.name(),
            operation.type_().clone(),
            transform_expression(
                operation.then(),
                &variables
                    .clone()
                    .into_iter()
                    .chain([(operation.name().into(), operation.type_().clone())])
                    .collect(),
            ),
        )
        .into(),
        Expression::TypeInformation(information) => {
            TypeInformation::new(information.index(), transform(information.variant())).into()
        }
        Expression::Variant(variant) => {
            Variant::new(variant.type_().clone(), transform(variant.payload())).into()
        }
        Expression::Boolean(_)
        | Expression::ByteString(_)
        | Expression::None
        | Expression::Number(_)
        | Expression::Variable(_) => expression.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test::{FunctionDefinitionFake, ModuleFake},
        types,
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn infer_empty_environment() {
        assert_eq!(
            transform_local_function_definition(
                &FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::Number)],
                    Type::Number,
                    42.0
                ),
                &Default::default()
            ),
            FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::Number)],
                Type::Number,
                42.0
            )
            .set_environment(vec![])
        );
    }

    #[test]
    fn infer_environment() {
        assert_eq!(
            transform_local_function_definition(
                &FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::Number)],
                    Type::Number,
                    Variable::new("y")
                ),
                &vec![("y".into(), Type::Number)].drain(..).collect()
            ),
            FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::Number)],
                Type::Number,
                Variable::new("y")
            )
            .set_environment(vec![Argument::new("y", Type::Number)])
        );
    }

    #[test]
    fn infer_environment_idempotently() {
        let variables = vec![("y".into(), Type::Number)].drain(..).collect();

        assert_eq!(
            transform_local_function_definition(
                &transform_local_function_definition(
                    &FunctionDefinition::new(
                        "f",
                        vec![Argument::new("x", Type::Number)],
                        Type::Number,
                        Variable::new("y")
                    ),
                    &variables
                ),
                &variables
            ),
            FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::Number)],
                Type::Number,
                Variable::new("y")
            )
            .set_environment(vec![Argument::new("y", Type::Number)])
        );
    }

    #[test]
    fn infer_environment_for_recursive_definition() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "a",
                    vec![],
                    Type::Number,
                    LetRecursive::new(
                        FunctionDefinition::new(
                            "f",
                            vec![Argument::new("x", Type::Number)],
                            Type::Number,
                            Call::new(
                                types::Function::new(vec![Type::Number], Type::Number),
                                Variable::new("f"),
                                vec![Variable::new("x").into()]
                            ),
                        ),
                        42.0
                    )
                )]),
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "a",
                vec![],
                Type::Number,
                LetRecursive::new(
                    FunctionDefinition::new(
                        "f",
                        vec![Argument::new("x", Type::Number)],
                        Type::Number,
                        Call::new(
                            types::Function::new(vec![Type::Number], Type::Number),
                            Variable::new("f"),
                            vec![Variable::new("x").into()]
                        ),
                    ),
                    42.0
                )
            )]),
        );
    }

    #[test]
    fn infer_environment_for_recursive_definition_shadowing_outer_variable() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "a",
                    vec![],
                    Type::Number,
                    LetRecursive::new(
                        FunctionDefinition::new(
                            "f",
                            vec![Argument::new("x", Type::Number)],
                            Type::Number,
                            Call::new(
                                types::Function::new(vec![Type::Number], Type::Number),
                                LetRecursive::new(
                                    FunctionDefinition::new(
                                        "f",
                                        vec![Argument::new("x", Type::Number)],
                                        Type::Number,
                                        Call::new(
                                            types::Function::new(vec![Type::Number], Type::Number),
                                            Variable::new("f"),
                                            vec![Variable::new("x").into()]
                                        )
                                    ),
                                    Variable::new("f")
                                ),
                                vec![Variable::new("x").into()]
                            )
                        ),
                        42.0
                    )
                )])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "a",
                vec![],
                Type::Number,
                LetRecursive::new(
                    FunctionDefinition::new(
                        "f",
                        vec![Argument::new("x", Type::Number)],
                        Type::Number,
                        Call::new(
                            types::Function::new(vec![Type::Number], Type::Number),
                            LetRecursive::new(
                                FunctionDefinition::new(
                                    "f",
                                    vec![Argument::new("x", Type::Number)],
                                    Type::Number,
                                    Call::new(
                                        types::Function::new(vec![Type::Number], Type::Number),
                                        Variable::new("f"),
                                        vec![Variable::new("x").into()]
                                    ),
                                )
                                .set_environment(vec![]),
                                Variable::new("f")
                            ),
                            vec![Variable::new("x").into()]
                        )
                    )
                    .set_environment(vec![]),
                    42.0
                )
            )])
        );
    }

    #[test]
    fn infer_environment_for_nested_function_definitions() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "a",
                    vec![],
                    Type::Number,
                    LetRecursive::new(
                        FunctionDefinition::new(
                            "f",
                            vec![Argument::new("x", Type::Number)],
                            Type::Number,
                            42.0
                        ),
                        LetRecursive::new(
                            FunctionDefinition::new(
                                "g",
                                vec![Argument::new("x", Type::Number)],
                                Type::Number,
                                Call::new(
                                    types::Function::new(vec![Type::Number], Type::Number),
                                    Variable::new("f"),
                                    vec![Variable::new("x").into()]
                                )
                            ),
                            42.0,
                        )
                    )
                )]),
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "a",
                vec![],
                Type::Number,
                LetRecursive::new(
                    FunctionDefinition::new(
                        "f",
                        vec![Argument::new("x", Type::Number)],
                        Type::Number,
                        42.0
                    ),
                    LetRecursive::new(
                        FunctionDefinition::new(
                            "g",
                            vec![Argument::new("x", Type::Number)],
                            Type::Number,
                            Call::new(
                                types::Function::new(vec![Type::Number], Type::Number),
                                Variable::new("f"),
                                vec![Variable::new("x").into()]
                            )
                        )
                        .set_environment(vec![Argument::new(
                            "f",
                            types::Function::new(vec![Type::Number], Type::Number)
                        )]),
                        42.0,
                    )
                )
            )])
        );
    }

    #[test]
    fn infer_environment_with_shadowed_variable() {
        assert_eq!(
            transform_local_function_definition(
                &FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::Number)],
                    Type::Number,
                    Variable::new("x")
                ),
                &vec![("x".into(), Type::Number)].drain(..).collect()
            ),
            FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::Number)],
                Type::Number,
                Variable::new("x")
            )
            .set_environment(vec![])
        );
    }
}
