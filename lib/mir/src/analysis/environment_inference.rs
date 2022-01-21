use super::free_variables::find_free_variables;
use crate::{ir::*, types::Type};
use std::collections::BTreeMap;

pub fn infer_environment(module: &Module) -> Module {
    Module::new(
        module.type_definitions().to_vec(),
        module.foreign_declarations().to_vec(),
        module.foreign_definitions().to_vec(),
        module.declarations().to_vec(),
        module
            .definitions()
            .iter()
            .map(infer_in_global_definition)
            .collect(),
    )
}

fn infer_in_global_definition(definition: &Definition) -> Definition {
    Definition::with_options(
        definition.name(),
        vec![],
        definition.arguments().to_vec(),
        infer_in_expression(
            definition.body(),
            &definition
                .arguments()
                .iter()
                .map(|argument| (argument.name().into(), argument.type_().clone()))
                .collect(),
        ),
        definition.result_type().clone(),
        definition.is_thunk(),
    )
}

fn infer_in_local_definition(
    definition: &Definition,
    variables: &BTreeMap<String, Type>,
) -> Definition {
    let local_variables = [(definition.name().into(), definition.type_().clone().into())]
        .into_iter()
        .chain(
            definition
                .arguments()
                .iter()
                .map(|argument| (argument.name().into(), argument.type_().clone())),
        )
        .collect::<BTreeMap<_, _>>();

    Definition::with_options(
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
        infer_in_expression(
            definition.body(),
            &variables
                .clone()
                .into_iter()
                .chain(local_variables)
                .collect(),
        ),
        definition.result_type().clone(),
        definition.is_thunk(),
    )
}

fn infer_in_expression(expression: &Expression, variables: &BTreeMap<String, Type>) -> Expression {
    match expression {
        Expression::ArithmeticOperation(operation) => {
            infer_in_arithmetic_operation(operation, variables).into()
        }
        Expression::Case(case) => infer_in_case(case, variables).into(),
        Expression::CloneVariables(clone) => infer_in_clone_variables(clone, variables).into(),
        Expression::ComparisonOperation(operation) => {
            infer_in_comparison_operation(operation, variables).into()
        }
        Expression::DropVariables(drop) => infer_in_drop_variables(drop, variables).into(),
        Expression::Call(call) => infer_in_call(call, variables).into(),
        Expression::If(if_) => infer_in_if(if_, variables).into(),
        Expression::Let(let_) => infer_in_let(let_, variables).into(),
        Expression::LetRecursive(let_) => infer_in_let_recursive(let_, variables).into(),
        Expression::Record(record) => infer_in_record(record, variables).into(),
        Expression::RecordField(field) => infer_in_record_field(field, variables).into(),
        Expression::TryOperation(operation) => infer_in_try_operation(operation, variables).into(),
        Expression::Variant(variant) => infer_in_variant(variant, variables).into(),
        Expression::Boolean(_)
        | Expression::ByteString(_)
        | Expression::None
        | Expression::Number(_)
        | Expression::Variable(_) => expression.clone(),
    }
}

fn infer_in_arithmetic_operation(
    operation: &ArithmeticOperation,
    variables: &BTreeMap<String, Type>,
) -> ArithmeticOperation {
    ArithmeticOperation::new(
        operation.operator(),
        infer_in_expression(operation.lhs(), variables),
        infer_in_expression(operation.rhs(), variables),
    )
}

fn infer_in_if(if_: &If, variables: &BTreeMap<String, Type>) -> If {
    If::new(
        infer_in_expression(if_.condition(), variables),
        infer_in_expression(if_.then(), variables),
        infer_in_expression(if_.else_(), variables),
    )
}

fn infer_in_case(case: &Case, variables: &BTreeMap<String, Type>) -> Case {
    Case::new(
        infer_in_expression(case.argument(), variables),
        case.alternatives()
            .iter()
            .map(|alternative| infer_in_alternative(alternative, variables))
            .collect(),
        case.default_alternative()
            .map(|alternative| infer_in_default_alternative(alternative, variables)),
    )
}

fn infer_in_alternative(
    alternative: &Alternative,
    variables: &BTreeMap<String, Type>,
) -> Alternative {
    let mut variables = variables.clone();

    variables.insert(alternative.name().into(), alternative.type_().clone());

    Alternative::new(
        alternative.type_().clone(),
        alternative.name(),
        infer_in_expression(alternative.expression(), &variables),
    )
}

fn infer_in_default_alternative(
    alternative: &DefaultAlternative,
    variables: &BTreeMap<String, Type>,
) -> DefaultAlternative {
    let mut variables = variables.clone();

    variables.insert(alternative.name().into(), Type::Variant);

    DefaultAlternative::new(
        alternative.name(),
        infer_in_expression(alternative.expression(), &variables),
    )
}

fn infer_in_clone_variables(
    clone: &CloneVariables,
    variables: &BTreeMap<String, Type>,
) -> CloneVariables {
    CloneVariables::new(
        clone.variables().clone(),
        infer_in_expression(clone.expression(), variables),
    )
}

fn infer_in_comparison_operation(
    operation: &ComparisonOperation,
    variables: &BTreeMap<String, Type>,
) -> ComparisonOperation {
    ComparisonOperation::new(
        operation.operator(),
        infer_in_expression(operation.lhs(), variables),
        infer_in_expression(operation.rhs(), variables),
    )
}

fn infer_in_drop_variables(
    drop: &DropVariables,
    variables: &BTreeMap<String, Type>,
) -> DropVariables {
    DropVariables::new(
        drop.variables().clone(),
        infer_in_expression(drop.expression(), variables),
    )
}

fn infer_in_call(call: &Call, variables: &BTreeMap<String, Type>) -> Call {
    Call::new(
        call.type_().clone(),
        infer_in_expression(call.function(), variables),
        call.arguments()
            .iter()
            .map(|argument| infer_in_expression(argument, variables))
            .collect(),
    )
}

fn infer_in_let(let_: &Let, variables: &BTreeMap<String, Type>) -> Let {
    Let::new(
        let_.name(),
        let_.type_().clone(),
        infer_in_expression(let_.bound_expression(), variables),
        infer_in_expression(
            let_.expression(),
            &variables
                .clone()
                .into_iter()
                .chain(vec![(let_.name().into(), let_.type_().clone())])
                .collect(),
        ),
    )
}

fn infer_in_let_recursive(let_: &LetRecursive, variables: &BTreeMap<String, Type>) -> LetRecursive {
    LetRecursive::new(
        infer_in_local_definition(let_.definition(), variables),
        infer_in_expression(
            let_.expression(),
            &variables
                .clone()
                .into_iter()
                .chain(vec![(
                    let_.definition().name().into(),
                    let_.definition().type_().clone().into(),
                )])
                .collect(),
        ),
    )
}

fn infer_in_record(record: &Record, variables: &BTreeMap<String, Type>) -> Record {
    Record::new(
        record.type_().clone(),
        record
            .fields()
            .iter()
            .map(|field| infer_in_expression(field, variables))
            .collect(),
    )
}

fn infer_in_record_field(field: &RecordField, variables: &BTreeMap<String, Type>) -> RecordField {
    RecordField::new(
        field.type_().clone(),
        field.index(),
        infer_in_expression(field.record(), variables),
    )
}

fn infer_in_try_operation(
    operation: &TryOperation,
    variables: &BTreeMap<String, Type>,
) -> TryOperation {
    TryOperation::new(
        infer_in_expression(operation.operand(), variables),
        operation.name(),
        operation.type_().clone(),
        infer_in_expression(
            operation.then(),
            &variables
                .clone()
                .into_iter()
                .chain(vec![(operation.name().into(), operation.type_().clone())])
                .collect(),
        ),
    )
}

fn infer_in_variant(variant: &Variant, variables: &BTreeMap<String, Type>) -> Variant {
    Variant::new(
        variant.type_().clone(),
        infer_in_expression(variant.payload(), variables),
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
            infer_in_local_definition(
                &Definition::new(
                    "f",
                    vec![Argument::new("x", Type::Number)],
                    42.0,
                    Type::Number
                ),
                &Default::default()
            ),
            Definition::with_environment(
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
            infer_in_local_definition(
                &Definition::new(
                    "f",
                    vec![Argument::new("x", Type::Number)],
                    Variable::new("y"),
                    Type::Number
                ),
                &vec![("y".into(), Type::Number)].drain(..).collect()
            ),
            Definition::with_environment(
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
            infer_in_local_definition(
                &infer_in_local_definition(
                    &Definition::new(
                        "f",
                        vec![Argument::new("x", Type::Number)],
                        Variable::new("y"),
                        Type::Number
                    ),
                    &variables
                ),
                &variables
            ),
            Definition::with_environment(
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
            infer_in_let_recursive(
                &LetRecursive::new(
                    Definition::new(
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
            &Definition::with_environment(
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
            infer_in_let_recursive(
                &LetRecursive::new(
                    Definition::new(
                        "f",
                        vec![Argument::new("x", Type::Number)],
                        Call::new(
                            types::Function::new(vec![Type::Number], Type::Number),
                            LetRecursive::new(
                                Definition::new(
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
            &Definition::with_environment(
                "f",
                vec![],
                vec![Argument::new("x", Type::Number)],
                Call::new(
                    types::Function::new(vec![Type::Number], Type::Number),
                    LetRecursive::new(
                        Definition::with_environment(
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
            infer_in_let_recursive(
                &LetRecursive::new(
                    Definition::new(
                        "f",
                        vec![Argument::new("x", Type::Number)],
                        42.0,
                        Type::Number
                    ),
                    LetRecursive::new(
                        Definition::new(
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
                Definition::with_options(
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
            infer_in_local_definition(
                &Definition::new(
                    "f",
                    vec![Argument::new("x", Type::Number)],
                    Variable::new("x"),
                    Type::Number
                ),
                &vec![("x".into(), Type::Number)].drain(..).collect()
            ),
            Definition::with_environment(
                "f",
                vec![],
                vec![Argument::new("x", Type::Number)],
                Variable::new("x"),
                Type::Number
            )
        );
    }
}
