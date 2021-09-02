use super::error::CompileError;
use crate::ast;
use hir::{ir, types};
use position::Position;

pub fn compile(module: &ast::Module) -> Result<ir::Module, CompileError> {
    Ok(ir::Module::new(
        module
            .type_definitions()
            .iter()
            .map(|definition| {
                ir::TypeDefinition::new(
                    definition.name(),
                    definition.name(),
                    definition.elements().to_vec(),
                    is_record_open(definition.elements()),
                    is_name_public(definition.name()),
                    false,
                    definition.position().clone(),
                )
            })
            .collect(),
        module
            .type_aliases()
            .iter()
            .map(|alias| {
                ir::TypeAlias::new(
                    alias.name(),
                    alias.name(),
                    alias.type_().clone(),
                    is_name_public(alias.name()),
                    false,
                    alias.position().clone(),
                )
            })
            .collect(),
        module
            .foreign_imports()
            .iter()
            .map(|import| {
                ir::ForeignDeclaration::new(
                    import.name(),
                    import.foreign_name(),
                    match import.calling_convention() {
                        ast::CallingConvention::C => ir::CallingConvention::C,
                        ast::CallingConvention::Native => ir::CallingConvention::Native,
                    },
                    import.type_().clone(),
                    import.position().clone(),
                )
            })
            .collect(),
        vec![],
        module
            .definitions()
            .iter()
            .map(|definition| compile_definition(definition))
            .collect::<Result<_, _>>()?,
        module.position().clone(),
    ))
}

fn compile_definition(definition: &ast::Definition) -> Result<ir::Definition, CompileError> {
    Ok(ir::Definition::new(
        definition.name(),
        definition.name(),
        compile_lambda(definition.lambda())?,
        definition.is_foreign(),
        is_name_public(definition.name()),
        definition.position().clone(),
    ))
}

fn compile_lambda(lambda: &ast::Lambda) -> Result<ir::Lambda, CompileError> {
    Ok(ir::Lambda::new(
        lambda
            .arguments()
            .iter()
            .map(|argument| ir::Argument::new(argument.name(), argument.type_().clone()))
            .collect(),
        lambda.result_type().clone(),
        compile_block(lambda.body())?,
        lambda.position().clone(),
    ))
}

fn compile_block(block: &ast::Block) -> Result<ir::Expression, CompileError> {
    let mut expression = compile_expression(block.expression())?;

    for statement in block.statements().iter().rev() {
        expression = ir::Let::new(
            statement.name().map(String::from),
            None,
            compile_expression(statement.expression())?,
            expression,
            statement.position().clone(),
        )
        .into();
    }

    Ok(expression)
}

fn compile_expression(expression: &ast::Expression) -> Result<ir::Expression, CompileError> {
    Ok(match expression {
        ast::Expression::BinaryOperation(operation) => {
            let lhs = compile_expression(operation.lhs())?;
            let rhs = compile_expression(operation.rhs())?;
            let position = operation.position().clone();

            match operation.operator() {
                ast::BinaryOperator::Add => {
                    ir::ArithmeticOperation::new(ir::ArithmeticOperator::Add, lhs, rhs, position)
                        .into()
                }
                ast::BinaryOperator::Subtract => ir::ArithmeticOperation::new(
                    ir::ArithmeticOperator::Subtract,
                    lhs,
                    rhs,
                    position,
                )
                .into(),
                ast::BinaryOperator::Multiply => ir::ArithmeticOperation::new(
                    ir::ArithmeticOperator::Multiply,
                    lhs,
                    rhs,
                    position,
                )
                .into(),
                ast::BinaryOperator::Divide => {
                    ir::ArithmeticOperation::new(ir::ArithmeticOperator::Divide, lhs, rhs, position)
                        .into()
                }

                ast::BinaryOperator::And => {
                    ir::BooleanOperation::new(ir::BooleanOperator::And, lhs, rhs, position).into()
                }
                ast::BinaryOperator::Or => {
                    ir::BooleanOperation::new(ir::BooleanOperator::Or, lhs, rhs, position).into()
                }

                ast::BinaryOperator::Equal => ir::EqualityOperation::new(
                    None,
                    ir::EqualityOperator::Equal,
                    lhs,
                    rhs,
                    position,
                )
                .into(),
                ast::BinaryOperator::NotEqual => ir::EqualityOperation::new(
                    None,
                    ir::EqualityOperator::NotEqual,
                    lhs,
                    rhs,
                    position,
                )
                .into(),

                ast::BinaryOperator::LessThan => {
                    ir::OrderOperation::new(ir::OrderOperator::LessThan, lhs, rhs, position).into()
                }
                ast::BinaryOperator::LessThanOrEqual => {
                    ir::OrderOperation::new(ir::OrderOperator::LessThanOrEqual, lhs, rhs, position)
                        .into()
                }
                ast::BinaryOperator::GreaterThan => {
                    ir::OrderOperation::new(ir::OrderOperator::GreaterThan, lhs, rhs, position)
                        .into()
                }
                ast::BinaryOperator::GreaterThanOrEqual => ir::OrderOperation::new(
                    ir::OrderOperator::GreaterThanOrEqual,
                    lhs,
                    rhs,
                    position,
                )
                .into(),
            }
        }
        ast::Expression::Boolean(boolean) => {
            ir::Boolean::new(boolean.value(), boolean.position().clone()).into()
        }
        ast::Expression::Call(call) => ir::Call::new(
            None,
            compile_expression(call.function())?,
            call.arguments()
                .iter()
                .map(|argument| compile_expression(argument))
                .collect::<Result<_, _>>()?,
            call.position().clone(),
        )
        .into(),
        ast::Expression::RecordDeconstruction(operation) => ir::RecordDeconstruction::new(
            None,
            compile_expression(operation.expression())?,
            operation.name(),
            operation.position().clone(),
        )
        .into(),
        ast::Expression::If(if_) => compile_if(if_.branches(), if_.else_(), if_.position())?.into(),
        ast::Expression::IfList(if_) => ir::IfList::new(
            None,
            compile_expression(if_.argument())?,
            if_.first_name(),
            if_.rest_name(),
            compile_block(if_.then())?,
            compile_block(if_.else_())?,
            if_.position().clone(),
        )
        .into(),
        ast::Expression::IfType(if_) => ir::IfType::new(
            if_.name(),
            compile_expression(if_.argument())?,
            if_.branches()
                .iter()
                .map(|branch| {
                    Ok(ir::IfTypeBranch::new(
                        branch.type_().clone(),
                        compile_block(branch.block())?,
                    ))
                })
                .collect::<Result<_, _>>()?,
            if_.else_()
                .map(|block| {
                    Ok(ir::ElseBranch::new(
                        None,
                        compile_block(block)?,
                        block.position().clone(),
                    ))
                })
                .transpose()?,
            if_.position().clone(),
        )
        .into(),
        ast::Expression::Lambda(lambda) => compile_lambda(lambda)?.into(),
        ast::Expression::List(list) => ir::List::new(
            list.type_().clone(),
            list.elements()
                .iter()
                .map(|element| {
                    Ok(match element {
                        ast::ListElement::Multiple(element) => {
                            ir::ListElement::Multiple(compile_expression(element)?)
                        }
                        ast::ListElement::Single(element) => {
                            ir::ListElement::Single(compile_expression(element)?)
                        }
                    })
                })
                .collect::<Result<_, _>>()?,
            list.position().clone(),
        )
        .into(),
        ast::Expression::None(none) => ir::None::new(none.position().clone()).into(),
        ast::Expression::Number(number) => {
            ir::Number::new(number.value(), number.position().clone()).into()
        }
        ast::Expression::Record(record) => {
            let elements = record
                .elements()
                .iter()
                .map(|element| {
                    Ok(ir::RecordElement::new(
                        element.name(),
                        compile_expression(element.expression())?,
                        element.position().clone(),
                    ))
                })
                .collect::<Result<_, _>>()?;

            if let Some(old_record) = record.record() {
                ir::RecordUpdate::new(
                    record.type_().clone(),
                    compile_expression(old_record)?,
                    elements,
                    record.position().clone(),
                )
                .into()
            } else {
                ir::RecordConstruction::new(
                    record.type_().clone(),
                    elements,
                    record.position().clone(),
                )
                .into()
            }
        }
        ast::Expression::String(string) => {
            ir::ByteString::new(string.value(), string.position().clone()).into()
        }
        ast::Expression::UnaryOperation(operation) => {
            let operand = compile_expression(operation.expression())?;

            match operation.operator() {
                ast::UnaryOperator::Not => {
                    ir::NotOperation::new(operand, operation.position().clone()).into()
                }
                ast::UnaryOperator::Try => {
                    ir::TryOperation::new(None, operand, operation.position().clone()).into()
                }
            }
        }
        ast::Expression::Variable(variable) => {
            ir::Variable::new(variable.name(), variable.position().clone()).into()
        }
    })
}

fn compile_if(
    branches: &[ast::IfBranch],
    else_: &ast::Block,
    position: &Position,
) -> Result<ir::If, CompileError> {
    Ok(match branches {
        [] => return Err(CompileError::TooFewBranchesInIf(position.clone())),
        [then] => ir::If::new(
            compile_expression(then.condition())?,
            compile_block(then.block())?,
            compile_block(else_)?,
            position.clone(),
        ),
        [then, ..] => ir::If::new(
            compile_expression(then.condition())?,
            compile_block(then.block())?,
            compile_if(&branches[1..], else_, position)?,
            position.clone(),
        ),
    })
}

fn is_record_open(elements: &[types::RecordElement]) -> bool {
    !elements.is_empty()
        && elements
            .iter()
            .all(|element| is_name_public(element.name()))
}

fn is_name_public(name: &str) -> bool {
    name.chars().next().unwrap().is_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hir::test::ModuleFake;
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    #[test]
    fn compile_empty_module() {
        assert_eq!(
            compile(&ast::Module::new(
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                Position::fake()
            )),
            Ok(ir::Module::empty())
        );
    }

    #[test]
    fn compile_module() {
        assert_eq!(
            compile(&ast::Module::new(
                vec![],
                vec![],
                vec![ast::TypeDefinition::new("Foo1", vec![], Position::fake())],
                vec![ast::TypeAlias::new(
                    "Foo2",
                    types::None::new(Position::fake()),
                    Position::fake(),
                )],
                vec![ast::Definition::new(
                    "Foo3",
                    ast::Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        ast::Block::new(vec![], ast::None::new(Position::fake()), Position::fake()),
                        Position::fake(),
                    ),
                    false,
                    Position::fake(),
                )],
                Position::fake(),
            )),
            Ok(ir::Module::empty()
                .set_type_definitions(vec![ir::TypeDefinition::new(
                    "Foo1",
                    "Foo1",
                    vec![],
                    false,
                    true,
                    false,
                    Position::fake()
                )])
                .set_type_aliases(vec![ir::TypeAlias::new(
                    "Foo2",
                    "Foo2",
                    types::None::new(Position::fake()),
                    true,
                    false,
                    Position::fake()
                )])
                .set_definitions(vec![ir::Definition::new(
                    "Foo3",
                    "Foo3",
                    ir::Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        ir::None::new(Position::fake()),
                        Position::fake(),
                    ),
                    false,
                    true,
                    Position::fake()
                )]))
        );
    }
}
