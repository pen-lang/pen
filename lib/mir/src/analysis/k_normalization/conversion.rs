use crate::{ir::*, types::Type};

pub fn convert(module: &Module) -> Module {
    Module::new(
        module.type_definitions().to_vec(),
        module.foreign_declarations().to_vec(),
        module.foreign_definitions().to_vec(),
        module.function_declarations().to_vec(),
        module
            .function_definitions()
            .iter()
            .map(convert_definition)
            .collect(),
    )
}

fn convert_definition(definition: &FunctionDefinition) -> FunctionDefinition {
    FunctionDefinition::with_options(
        definition.name(),
        definition.environment().to_vec(),
        definition.arguments().to_vec(),
        convert_expression(definition.body(), &|expression| expression),
        definition.result_type().clone(),
        definition.is_thunk(),
    )
}

fn convert_expression(
    expression: &Expression,
    callback: &dyn Fn(Expression) -> Expression,
) -> Expression {
    let (expression, type_): (Expression, Type) = match expression {
        Expression::ArithmeticOperation(operation) => (
            convert_expression(operation.lhs(), &|lhs| {
                convert_expression(operation.rhs(), &|rhs| {
                    ArithmeticOperation::new(operation.operator(), lhs.clone(), rhs.clone()).into()
                })
            }),
            Type::Number,
        ),
        Expression::Call(_) => todo!(),
        Expression::Case(_) => todo!(),
        Expression::CloneVariables(_) => todo!(),
        Expression::ComparisonOperation(_) => todo!(),
        Expression::DiscardHeap(_) => todo!(),
        Expression::DropVariables(_) => todo!(),
        Expression::If(_) => todo!(),
        Expression::Let(_) => todo!(),
        Expression::LetRecursive(_) => todo!(),
        Expression::Record(_) => todo!(),
        Expression::RecordField(_) => todo!(),
        Expression::RetainHeap(_) => todo!(),
        Expression::ReuseRecord(_) => todo!(),
        Expression::TryOperation(_) => todo!(),
        Expression::Variant(_) => todo!(),
        Expression::Boolean(_)
        | Expression::ByteString(_)
        | Expression::None
        | Expression::Number(_)
        | Expression::Variable(_) => return callback(expression.clone()),
    };

    // TODO
    let name = "x";

    Let::new(
        name,
        type_,
        expression,
        callback(Variable::new(name).into()),
    )
    .into()
}
