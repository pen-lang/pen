use super::free_variables::find_free_variables;
use crate::{ir::*, types::Type};
use std::collections::HashMap;

pub fn infer_environment(module: &Module) -> Module {
    Module::new(
        module.type_definitions().to_vec(),
        module.foreign_declarations().to_vec(),
        module.foreign_definitions().to_vec(),
        module.declarations().to_vec(),
        module
            .definitions()
            .iter()
            .map(|definition| infer_in_global_definition(definition))
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
    variables: &HashMap<String, Type>,
) -> Definition {
    Definition::with_options(
        definition.name(),
        find_free_variables(definition.body())
            .iter()
            .filter_map(|name| {
                variables
                    .get(name)
                    .map(|type_| Argument::new(name, type_.clone()))
            })
            .collect(),
        definition.arguments().to_vec(),
        infer_in_expression(
            definition.body(),
            &variables
                .clone()
                .drain()
                .chain(vec![(
                    definition.name().into(),
                    definition.type_().clone().into(),
                )])
                .chain(
                    definition
                        .arguments()
                        .iter()
                        .map(|argument| (argument.name().into(), argument.type_().clone())),
                )
                .collect(),
        ),
        definition.result_type().clone(),
        definition.is_thunk(),
    )
}

fn infer_in_expression(expression: &Expression, variables: &HashMap<String, Type>) -> Expression {
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
        Expression::FunctionApplication(application) => {
            infer_in_function_application(application, variables).into()
        }
        Expression::If(if_) => infer_in_if(if_, variables).into(),
        Expression::Let(let_) => infer_in_let(let_, variables).into(),
        Expression::LetRecursive(let_) => infer_in_let_recursive(let_, variables).into(),
        Expression::Record(record) => infer_in_record(record, variables).into(),
        Expression::RecordElement(element) => infer_in_record_element(element, variables).into(),
        Expression::Variant(variant) => infer_in_variant(variant, variables).into(),
        Expression::Boolean(_)
        | Expression::ByteString(_)
        | Expression::Number(_)
        | Expression::Variable(_) => expression.clone(),
    }
}

fn infer_in_arithmetic_operation(
    operation: &ArithmeticOperation,
    variables: &HashMap<String, Type>,
) -> ArithmeticOperation {
    ArithmeticOperation::new(
        operation.operator(),
        infer_in_expression(operation.lhs(), variables),
        infer_in_expression(operation.rhs(), variables),
    )
}

fn infer_in_if(if_: &If, variables: &HashMap<String, Type>) -> If {
    If::new(
        infer_in_expression(if_.condition(), variables),
        infer_in_expression(if_.then(), variables),
        infer_in_expression(if_.else_(), variables),
    )
}

fn infer_in_case(case: &Case, variables: &HashMap<String, Type>) -> Case {
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
    variables: &HashMap<String, Type>,
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
    variables: &HashMap<String, Type>,
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
    variables: &HashMap<String, Type>,
) -> CloneVariables {
    CloneVariables::new(
        clone.variables().clone(),
        infer_in_expression(clone.expression(), variables),
    )
}

fn infer_in_comparison_operation(
    operation: &ComparisonOperation,
    variables: &HashMap<String, Type>,
) -> ComparisonOperation {
    ComparisonOperation::new(
        operation.operator(),
        infer_in_expression(operation.lhs(), variables),
        infer_in_expression(operation.rhs(), variables),
    )
}

fn infer_in_drop_variables(
    drop: &DropVariables,
    variables: &HashMap<String, Type>,
) -> DropVariables {
    DropVariables::new(
        drop.variables().clone(),
        infer_in_expression(drop.expression(), variables),
    )
}

fn infer_in_function_application(
    application: &FunctionApplication,
    variables: &HashMap<String, Type>,
) -> FunctionApplication {
    FunctionApplication::new(
        application.type_().clone(),
        infer_in_expression(application.function(), variables),
        infer_in_expression(application.argument(), variables),
    )
}

fn infer_in_let(let_: &Let, variables: &HashMap<String, Type>) -> Let {
    Let::new(
        let_.name(),
        let_.type_().clone(),
        infer_in_expression(let_.bound_expression(), variables),
        infer_in_expression(
            let_.expression(),
            &variables
                .clone()
                .drain()
                .chain(vec![(let_.name().into(), let_.type_().clone())])
                .collect(),
        ),
    )
}

fn infer_in_let_recursive(let_: &LetRecursive, variables: &HashMap<String, Type>) -> LetRecursive {
    LetRecursive::new(
        infer_in_local_definition(let_.definition(), &variables),
        infer_in_expression(
            let_.expression(),
            &variables
                .clone()
                .drain()
                .chain(vec![(
                    let_.definition().name().into(),
                    let_.definition().type_().clone().into(),
                )])
                .collect(),
        ),
    )
}

fn infer_in_record(record: &Record, variables: &HashMap<String, Type>) -> Record {
    Record::new(
        record.type_().clone(),
        record
            .elements()
            .iter()
            .map(|element| infer_in_expression(element, variables))
            .collect(),
    )
}

fn infer_in_record_element(
    element: &RecordElement,
    variables: &HashMap<String, Type>,
) -> RecordElement {
    RecordElement::new(
        element.type_().clone(),
        element.index(),
        infer_in_expression(element.record(), variables),
    )
}

fn infer_in_variant(variant: &Variant, variables: &HashMap<String, Type>) -> Variant {
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
                        FunctionApplication::new(
                            types::Function::new(Type::Number, Type::Number),
                            Variable::new("f"),
                            Variable::new("x")
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
                FunctionApplication::new(
                    types::Function::new(Type::Number, Type::Number),
                    Variable::new("f"),
                    Variable::new("x")
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
                            FunctionApplication::new(
                                types::Function::new(Type::Number, Type::Number),
                                Variable::new("f"),
                                Variable::new("x")
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
                        types::Function::new(Type::Number, Type::Number)
                    )],
                    vec![Argument::new("x", Type::Number)],
                    FunctionApplication::new(
                        types::Function::new(Type::Number, Type::Number),
                        Variable::new("f"),
                        Variable::new("x")
                    ),
                    Type::Number,
                    false
                ),
                42.0,
            )
            .into()
        );
    }
}
