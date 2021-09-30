use crate::{ir::*, types::Type};
use std::collections::*;

pub fn collect_variant_types(module: &Module) -> HashSet<Type> {
    module
        .definitions()
        .iter()
        .flat_map(collect_from_definition)
        .collect()
}

fn collect_from_definition(definition: &Definition) -> HashSet<Type> {
    collect_from_expression(definition.body())
}

fn collect_from_expression(expression: &Expression) -> HashSet<Type> {
    match expression {
        Expression::ArithmeticOperation(operation) => collect_from_expression(operation.lhs())
            .drain()
            .chain(collect_from_expression(operation.rhs()))
            .collect(),
        Expression::Case(case) => collect_from_case(case),
        Expression::CloneVariables(clone) => collect_from_expression(clone.expression()),
        Expression::ComparisonOperation(operation) => collect_from_expression(operation.lhs())
            .drain()
            .chain(collect_from_expression(operation.rhs()))
            .collect(),
        Expression::DropVariables(drop) => collect_from_expression(drop.expression()),
        Expression::Call(call) => collect_from_expression(call.function())
            .drain()
            .chain(call.arguments().iter().flat_map(collect_from_expression))
            .collect(),
        Expression::If(if_) => collect_from_expression(if_.condition())
            .drain()
            .chain(collect_from_expression(if_.then()))
            .chain(collect_from_expression(if_.else_()))
            .collect(),
        Expression::Let(let_) => collect_from_expression(let_.bound_expression())
            .drain()
            .chain(collect_from_expression(let_.expression()))
            .collect(),
        Expression::LetRecursive(let_) => collect_from_definition(let_.definition())
            .into_iter()
            .chain(collect_from_expression(let_.expression()))
            .collect(),
        Expression::Record(record) => record
            .elements()
            .iter()
            .flat_map(collect_from_expression)
            .collect(),
        Expression::RecordElement(element) => collect_from_expression(element.record()),
        Expression::TryOperation(operation) => vec![operation.type_().clone()]
            .into_iter()
            .chain(collect_from_expression(operation.operand()))
            .chain(collect_from_expression(operation.then()))
            .collect(),
        Expression::Variant(variant) => vec![variant.type_().clone()]
            .into_iter()
            .chain(collect_from_expression(variant.payload()))
            .collect(),
        Expression::Boolean(_)
        | Expression::ByteString(_)
        | Expression::None
        | Expression::Number(_)
        | Expression::Variable(_) => Default::default(),
    }
}

fn collect_from_case(case: &Case) -> HashSet<Type> {
    collect_from_expression(case.argument())
        .into_iter()
        .chain(case.alternatives().iter().flat_map(|alternative| {
            vec![alternative.type_().clone()]
                .into_iter()
                .chain(collect_from_expression(alternative.expression()))
        }))
        .chain(
            case.default_alternative()
                .map(|alternative| collect_from_expression(alternative.expression()))
                .unwrap_or_default(),
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Type;

    #[test]
    fn collect_from_case_argument() {
        assert_eq!(
            collect_variant_types(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![],
                vec![Definition::new(
                    "f",
                    vec![],
                    Case::new(Variant::new(Type::Number, Variable::new("x")), vec![], None),
                    Type::None,
                )],
            )),
            vec![Type::Number].into_iter().collect()
        );
    }

    #[test]
    fn collect_from_try_operation_operand() {
        assert_eq!(
            collect_variant_types(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![],
                vec![Definition::new(
                    "f",
                    vec![Argument::new("x", Type::Variant)],
                    TryOperation::new(
                        Variable::new("x"),
                        "error",
                        Type::Number,
                        Variable::new("error"),
                    ),
                    Type::None,
                )],
            )),
            vec![Type::Number].into_iter().collect()
        );
    }

    #[test]
    fn collect_from_try_operation_then_expression() {
        assert_eq!(
            collect_variant_types(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![],
                vec![Definition::new(
                    "f",
                    vec![Argument::new("x", Type::Variant)],
                    TryOperation::new(
                        Variable::new("x"),
                        "error",
                        Type::Number,
                        Variant::new(Type::Boolean, Variable::new("error")),
                    ),
                    Type::None,
                )],
            )),
            vec![Type::Boolean, Type::Number].into_iter().collect()
        );
    }
}
