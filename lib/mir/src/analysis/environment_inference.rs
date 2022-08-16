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
    )
}

fn transform_global_function_definition(definition: &FunctionDefinition) -> FunctionDefinition {
    FunctionDefinition::with_options(
        definition.name(),
        vec![],
        definition.arguments().to_vec(),
        transform_expression(
            definition.body(),
            &definition
                .arguments()
                .iter()
                .map(|argument| (argument.name().into(), argument.type_().clone()))
                .collect(),
        ),
        definition.result_type().clone(),
        definition.is_public(),
        definition.is_thunk(),
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
        transform_expression(
            definition.body(),
            &variables
                .clone()
                .into_iter()
                .chain(local_variables)
                .collect(),
        ),
        definition.result_type().clone(),
        definition.is_public(),
        definition.is_thunk(),
    )
}

fn transform_expression(
    expression: &Expression,
    variables: &FnvHashMap<String, Type>,
) -> Expression {
    match expression {
        Expression::ArithmeticOperation(operation) => {
            transform_arithmetic_operation(operation, variables).into()
        }
        Expression::Case(case) => transform_case(case, variables).into(),
        Expression::CloneVariables(clone) => transform_clone_variables(clone, variables).into(),
        Expression::ComparisonOperation(operation) => {
            transform_comparison_operation(operation, variables).into()
        }
        Expression::DropVariables(drop) => transform_drop_variables(drop, variables).into(),
        Expression::Call(call) => transform_call(call, variables).into(),
        Expression::If(if_) => transform_if(if_, variables).into(),
        Expression::Let(let_) => transform_let(let_, variables).into(),
        Expression::LetRecursive(let_) => transform_let_recursive(let_, variables).into(),
        Expression::Synchronize(synchronize) => Synchronize::new(
            synchronize.type_().clone(),
            transform_expression(synchronize.expression(), variables),
        )
        .into(),
        Expression::Record(record) => transform_record(record, variables).into(),
        Expression::RecordField(field) => transform_record_field(field, variables).into(),
        Expression::RecordUpdate(update) => transform_record_update(update, variables).into(),
        Expression::TryOperation(operation) => transform_try_operation(operation, variables).into(),
        Expression::Variant(variant) => transform_variant(variant, variables).into(),
        Expression::Boolean(_)
        | Expression::ByteString(_)
        | Expression::None
        | Expression::Number(_)
        | Expression::Variable(_) => expression.clone(),
    }
}

fn transform_arithmetic_operation(
    operation: &ArithmeticOperation,
    variables: &FnvHashMap<String, Type>,
) -> ArithmeticOperation {
    ArithmeticOperation::new(
        operation.operator(),
        transform_expression(operation.lhs(), variables),
        transform_expression(operation.rhs(), variables),
    )
}

fn transform_if(if_: &If, variables: &FnvHashMap<String, Type>) -> If {
    If::new(
        transform_expression(if_.condition(), variables),
        transform_expression(if_.then(), variables),
        transform_expression(if_.else_(), variables),
    )
}

fn transform_case(case: &Case, variables: &FnvHashMap<String, Type>) -> Case {
    Case::new(
        transform_expression(case.argument(), variables),
        case.alternatives()
            .iter()
            .map(|alternative| transform_alternative(alternative, variables))
            .collect(),
        case.default_alternative()
            .map(|alternative| transform_default_alternative(alternative, variables)),
    )
}

fn transform_alternative(
    alternative: &Alternative,
    variables: &FnvHashMap<String, Type>,
) -> Alternative {
    let mut variables = variables.clone();

    variables.insert(alternative.name().into(), alternative.type_().clone());

    Alternative::new(
        alternative.types().to_vec(),
        alternative.name(),
        transform_expression(alternative.expression(), &variables),
    )
}

fn transform_default_alternative(
    alternative: &DefaultAlternative,
    variables: &FnvHashMap<String, Type>,
) -> DefaultAlternative {
    let mut variables = variables.clone();

    variables.insert(alternative.name().into(), Type::Variant);

    DefaultAlternative::new(
        alternative.name(),
        transform_expression(alternative.expression(), &variables),
    )
}

fn transform_clone_variables(
    clone: &CloneVariables,
    variables: &FnvHashMap<String, Type>,
) -> CloneVariables {
    CloneVariables::new(
        clone.variables().clone(),
        transform_expression(clone.expression(), variables),
    )
}

fn transform_comparison_operation(
    operation: &ComparisonOperation,
    variables: &FnvHashMap<String, Type>,
) -> ComparisonOperation {
    ComparisonOperation::new(
        operation.operator(),
        transform_expression(operation.lhs(), variables),
        transform_expression(operation.rhs(), variables),
    )
}

fn transform_drop_variables(
    drop: &DropVariables,
    variables: &FnvHashMap<String, Type>,
) -> DropVariables {
    DropVariables::new(
        drop.variables().clone(),
        transform_expression(drop.expression(), variables),
    )
}

fn transform_call(call: &Call, variables: &FnvHashMap<String, Type>) -> Call {
    Call::new(
        call.type_().clone(),
        transform_expression(call.function(), variables),
        call.arguments()
            .iter()
            .map(|argument| transform_expression(argument, variables))
            .collect(),
    )
}

fn transform_let(let_: &Let, variables: &FnvHashMap<String, Type>) -> Let {
    Let::new(
        let_.name(),
        let_.type_().clone(),
        transform_expression(let_.bound_expression(), variables),
        transform_expression(
            let_.expression(),
            &variables
                .clone()
                .into_iter()
                .chain([(let_.name().into(), let_.type_().clone())])
                .collect(),
        ),
    )
}

fn transform_let_recursive(
    let_: &LetRecursive,
    variables: &FnvHashMap<String, Type>,
) -> LetRecursive {
    LetRecursive::new(
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
}

fn transform_record(record: &Record, variables: &FnvHashMap<String, Type>) -> Record {
    Record::new(
        record.type_().clone(),
        record
            .fields()
            .iter()
            .map(|field| transform_expression(field, variables))
            .collect(),
    )
}

fn transform_record_update(
    update: &RecordUpdate,
    variables: &FnvHashMap<String, Type>,
) -> RecordUpdate {
    RecordUpdate::new(
        update.type_().clone(),
        transform_expression(update.record(), variables),
        update
            .fields()
            .iter()
            .map(|field| {
                RecordUpdateField::new(
                    field.index(),
                    transform_expression(field.expression(), variables),
                )
            })
            .collect(),
    )
}

fn transform_record_field(
    field: &RecordField,
    variables: &FnvHashMap<String, Type>,
) -> RecordField {
    RecordField::new(
        field.type_().clone(),
        field.index(),
        transform_expression(field.record(), variables),
    )
}

fn transform_try_operation(
    operation: &TryOperation,
    variables: &FnvHashMap<String, Type>,
) -> TryOperation {
    TryOperation::new(
        transform_expression(operation.operand(), variables),
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
}

fn transform_variant(variant: &Variant, variables: &FnvHashMap<String, Type>) -> Variant {
    Variant::new(
        variant.type_().clone(),
        transform_expression(variant.payload(), variables),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types;
    use pretty_assertions::assert_eq;

    #[test]
    fn infer_empty_environment() {
        assert_eq!(
            transform_local_function_definition(
                &FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::Number)],
                    42.0,
                    Type::Number
                ),
                &Default::default()
            ),
            FunctionDefinition::with_environment(
                "f",
                vec![],
                vec![Argument::new("x", Type::Number)],
                42.0,
                Type::Number
            )
        );
    }

    #[test]
    fn infer_environment() {
        assert_eq!(
            transform_local_function_definition(
                &FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::Number)],
                    Variable::new("y"),
                    Type::Number
                ),
                &vec![("y".into(), Type::Number)].drain(..).collect()
            ),
            FunctionDefinition::with_environment(
                "f",
                vec![Argument::new("y", Type::Number)],
                vec![Argument::new("x", Type::Number)],
                Variable::new("y"),
                Type::Number
            )
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
                        Variable::new("y"),
                        Type::Number
                    ),
                    &variables
                ),
                &variables
            ),
            FunctionDefinition::with_environment(
                "f",
                vec![Argument::new("y", Type::Number)],
                vec![Argument::new("x", Type::Number)],
                Variable::new("y"),
                Type::Number
            )
        );
    }

    #[test]
    fn infer_environment_for_recursive_definition() {
        assert_eq!(
            transform_let_recursive(
                &LetRecursive::new(
                    FunctionDefinition::new(
                        "f",
                        vec![Argument::new("x", Type::Number)],
                        Call::new(
                            types::Function::new(vec![Type::Number], Type::Number),
                            Variable::new("f"),
                            vec![Variable::new("x").into()]
                        ),
                        Type::Number
                    ),
                    Expression::Number(42.0)
                ),
                &Default::default(),
            )
            .definition(),
            &FunctionDefinition::with_environment(
                "f",
                vec![],
                vec![Argument::new("x", Type::Number)],
                Call::new(
                    types::Function::new(vec![Type::Number], Type::Number),
                    Variable::new("f"),
                    vec![Variable::new("x").into()]
                ),
                Type::Number
            )
        );
    }

    #[test]
    fn infer_environment_for_recursive_definition_shadowing_outer_variable() {
        assert_eq!(
            transform_let_recursive(
                &LetRecursive::new(
                    FunctionDefinition::new(
                        "f",
                        vec![Argument::new("x", Type::Number)],
                        Call::new(
                            types::Function::new(vec![Type::Number], Type::Number),
                            LetRecursive::new(
                                FunctionDefinition::new(
                                    "f",
                                    vec![Argument::new("x", Type::Number)],
                                    Call::new(
                                        types::Function::new(vec![Type::Number], Type::Number),
                                        Variable::new("f"),
                                        vec![Variable::new("x").into()]
                                    ),
                                    Type::Number
                                ),
                                Variable::new("f")
                            ),
                            vec![Variable::new("x").into()]
                        ),
                        Type::Number
                    ),
                    Expression::Number(42.0)
                ),
                &Default::default(),
            )
            .definition(),
            &FunctionDefinition::with_environment(
                "f",
                vec![],
                vec![Argument::new("x", Type::Number)],
                Call::new(
                    types::Function::new(vec![Type::Number], Type::Number),
                    LetRecursive::new(
                        FunctionDefinition::with_environment(
                            "f",
                            vec![],
                            vec![Argument::new("x", Type::Number)],
                            Call::new(
                                types::Function::new(vec![Type::Number], Type::Number),
                                Variable::new("f"),
                                vec![Variable::new("x").into()]
                            ),
                            Type::Number
                        ),
                        Variable::new("f")
                    ),
                    vec![Variable::new("x").into()]
                ),
                Type::Number
            )
        );
    }

    #[test]
    fn infer_environment_for_nested_function_definitions() {
        assert_eq!(
            transform_let_recursive(
                &LetRecursive::new(
                    FunctionDefinition::new(
                        "f",
                        vec![Argument::new("x", Type::Number)],
                        42.0,
                        Type::Number
                    ),
                    LetRecursive::new(
                        FunctionDefinition::new(
                            "g",
                            vec![Argument::new("x", Type::Number)],
                            Call::new(
                                types::Function::new(vec![Type::Number], Type::Number),
                                Variable::new("f"),
                                vec![Variable::new("x").into()]
                            ),
                            Type::Number
                        ),
                        42.0,
                    )
                ),
                &Default::default(),
            )
            .expression(),
            &LetRecursive::new(
                FunctionDefinition::with_options(
                    "g",
                    vec![Argument::new(
                        "f",
                        types::Function::new(vec![Type::Number], Type::Number)
                    )],
                    vec![Argument::new("x", Type::Number)],
                    Call::new(
                        types::Function::new(vec![Type::Number], Type::Number),
                        Variable::new("f"),
                        vec![Variable::new("x").into()]
                    ),
                    Type::Number,
                    false
                ),
                42.0,
            )
            .into()
        );
    }

    #[test]
    fn infer_environment_with_shadowed_variable() {
        assert_eq!(
            transform_local_function_definition(
                &FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::Number)],
                    Variable::new("x"),
                    Type::Number
                ),
                &vec![("x".into(), Type::Number)].drain(..).collect()
            ),
            FunctionDefinition::with_environment(
                "f",
                vec![],
                vec![Argument::new("x", Type::Number)],
                Variable::new("x"),
                Type::Number
            )
        );
    }
}
