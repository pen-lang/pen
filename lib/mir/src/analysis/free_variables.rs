use crate::ir::*;
use fnv::FnvHashSet;

pub fn find_free_variables(expression: &Expression) -> FnvHashSet<String> {
    find_in_expression(expression)
}

fn find_in_expression(expression: &Expression) -> FnvHashSet<String> {
    match expression {
        Expression::ArithmeticOperation(operation) => find_in_expression(operation.lhs())
            .into_iter()
            .chain(find_in_expression(operation.rhs()))
            .collect(),
        Expression::BorrowRecordField(_) => todo!(),
        Expression::Case(case) => find_in_case(case),
        Expression::CloneVariables(clone) => find_in_expression(clone.expression()),
        Expression::ComparisonOperation(operation) => find_in_expression(operation.lhs())
            .into_iter()
            .chain(find_in_expression(operation.rhs()))
            .collect(),
        Expression::DiscardHeap(discard) => find_in_expression(discard.expression()),
        Expression::DropVariables(drop) => find_in_drop_variables(drop),
        Expression::Call(call) => find_in_expression(call.function())
            .into_iter()
            .chain(call.arguments().iter().flat_map(find_in_expression))
            .collect(),
        Expression::If(if_) => find_in_expression(if_.condition())
            .into_iter()
            .chain(find_in_expression(if_.then()))
            .chain(find_in_expression(if_.else_()))
            .collect(),
        Expression::LetRecursive(let_) => find_in_definition(let_.definition())
            .into_iter()
            .chain(find_in_expression(let_.expression()))
            .filter(|variable| variable != let_.definition().name())
            .collect(),
        Expression::Let(let_) => find_in_expression(let_.bound_expression())
            .into_iter()
            .chain(
                find_in_expression(let_.expression())
                    .into_iter()
                    .filter(|variable| variable != let_.name()),
            )
            .collect(),
        Expression::Record(record) => find_in_record(record),
        Expression::RecordField(field) => find_in_expression(field.record()),
        Expression::ReuseRecord(record) => find_in_record(record.record()),
        Expression::RetainHeap(reuse) => find_in_drop_variables(reuse.drop()),
        Expression::TryOperation(operation) => find_in_expression(operation.operand())
            .into_iter()
            .chain(
                find_in_expression(operation.then())
                    .into_iter()
                    .filter(|variable| variable != operation.name()),
            )
            .collect(),
        Expression::Variable(variable) => [variable.name().into()].into_iter().collect(),
        Expression::Variant(variant) => find_in_expression(variant.payload()),
        Expression::Boolean(_)
        | Expression::ByteString(_)
        | Expression::None
        | Expression::Number(_) => FnvHashSet::default(),
    }
}

fn find_in_case(case: &Case) -> FnvHashSet<String> {
    find_in_expression(case.argument())
        .into_iter()
        .chain(case.alternatives().iter().flat_map(|alternative| {
            find_in_expression(alternative.expression())
                .into_iter()
                .filter(|variable| variable != alternative.name())
                .collect::<FnvHashSet<_>>()
        }))
        .chain(
            case.default_alternative()
                .into_iter()
                .flat_map(|alternative| {
                    find_in_expression(alternative.expression())
                        .into_iter()
                        .filter(|variable| variable != alternative.name())
                        .collect::<FnvHashSet<_>>()
                }),
        )
        .collect()
}

fn find_in_drop_variables(drop: &DropVariables) -> FnvHashSet<String> {
    find_in_expression(drop.expression())
}

fn find_in_record(record: &Record) -> FnvHashSet<String> {
    record
        .fields()
        .iter()
        .flat_map(find_in_expression)
        .collect()
}

fn find_in_definition(definition: &FunctionDefinition) -> FnvHashSet<String> {
    find_in_expression(definition.body())
        .into_iter()
        .filter(|variable| {
            variable != definition.name()
                && definition
                    .arguments()
                    .iter()
                    .all(|argument| variable != argument.name())
        })
        .collect()
}
