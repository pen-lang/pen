use crate::{ir::*, types::Type};
use std::collections::*;

pub fn collect_variant_types(module: &Module) -> HashSet<Type> {
    module
        .definitions()
        .iter()
        .flat_map(|definition| collect_from_definition(definition))
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
        Expression::FunctionApplication(application) => {
            collect_from_expression(application.function())
                .drain()
                .chain(collect_from_expression(application.argument()))
                .collect()
        }
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
        Expression::Variant(variant) => vec![variant.type_().clone()]
            .into_iter()
            .chain(collect_from_expression(variant.payload()))
            .collect(),
        Expression::Boolean(_)
        | Expression::ByteString(_)
        | Expression::Number(_)
        | Expression::Variable(_) => Default::default(),
    }
}

fn collect_from_case(case: &Case) -> HashSet<Type> {
    case.alternatives()
        .iter()
        .flat_map(|alternative| {
            vec![alternative.type_().clone()]
                .into_iter()
                .chain(collect_from_expression(alternative.expression()))
        })
        .chain(
            case.default_alternative()
                .map(|alternative| collect_from_expression(alternative.expression()))
                .unwrap_or_default(),
        )
        .collect()
}
