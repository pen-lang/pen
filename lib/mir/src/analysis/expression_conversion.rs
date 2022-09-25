use crate::ir::*;

pub fn transform(module: &Module, convert: impl Fn(&Expression) -> Expression) -> Module {
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
                    transform_function_definition(definition.definition(), &convert),
                    definition.is_public(),
                )
            })
            .collect(),
    )
}

fn transform_function_definition(
    definition: &FunctionDefinition,
    convert: &dyn Fn(&Expression) -> Expression,
) -> FunctionDefinition {
    FunctionDefinition::with_options(
        definition.name(),
        definition.environment().to_vec(),
        definition.arguments().to_vec(),
        definition.result_type().clone(),
        transform_expression(definition.body(), convert),
        definition.is_thunk(),
    )
}

fn transform_expression(
    expression: &Expression,
    convert: &dyn Fn(&Expression) -> Expression,
) -> Expression {
    let transform = |expression| transform_expression(expression, convert);

    convert(&match expression {
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
                    Alternative::new(
                        alternative.types().to_vec(),
                        alternative.name(),
                        transform(alternative.expression()),
                    )
                })
                .collect(),
            case.default_alternative().map(|alternative| {
                DefaultAlternative::new(alternative.name(), transform(alternative.expression()))
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
        Expression::Let(let_) => Let::new(
            let_.name(),
            let_.type_().clone(),
            transform(let_.bound_expression()),
            transform(let_.expression()),
        )
        .into(),
        Expression::LetRecursive(let_) => LetRecursive::new(
            transform_function_definition(let_.definition(), convert),
            transform(let_.expression()),
        )
        .into(),
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
        Expression::TryOperation(operation) => TryOperation::new(
            transform(operation.operand()),
            operation.name(),
            operation.type_().clone(),
            transform(operation.then()),
        )
        .into(),
        Expression::TypeInformation(_) => todo!(),
        Expression::Variant(variant) => {
            Variant::new(variant.type_().clone(), transform(variant.payload())).into()
        }
        Expression::Boolean(_)
        | Expression::ByteString(_)
        | Expression::None
        | Expression::Number(_)
        | Expression::Variable(_) => expression.clone(),
    })
}
