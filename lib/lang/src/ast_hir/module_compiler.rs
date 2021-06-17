use super::error::CompileError;
use crate::{ast, hir, interface, position::Position, types};

pub fn compile(
    module: &ast::Module,
    module_interfaces: &[interface::Module],
) -> Result<hir::Module, CompileError> {
    Ok(hir::Module::new(
        module
            .type_definitions()
            .iter()
            .map(|definition| {
                hir::TypeDefinition::new(
                    definition.name(),
                    definition.elements().to_vec(),
                    is_record_open(definition.elements()),
                    false,
                    true,
                    definition.position().clone(),
                )
            })
            .collect(),
        module
            .type_aliases()
            .iter()
            .map(|alias| hir::TypeAlias::new(alias.name(), alias.type_().clone(), false, true))
            .collect(),
        module_interfaces
            .iter()
            .flat_map(|interface| interface.declarations())
            .map(|declaration| {
                hir::Declaration::new(
                    declaration.name(),
                    declaration.type_().clone(),
                    declaration.position().clone(),
                )
            })
            .collect(),
        module
            .definitions()
            .iter()
            .map(|definition| compile_definition(definition))
            .collect::<Result<_, _>>()?,
    ))
}

fn compile_definition(definition: &ast::Definition) -> Result<hir::Definition, CompileError> {
    Ok(hir::Definition::new(
        definition.name(),
        compile_lambda(definition.lambda())?,
        is_name_public(definition.name()),
        definition.position().clone(),
    ))
}

fn compile_lambda(lambda: &ast::Lambda) -> Result<hir::Lambda, CompileError> {
    Ok(hir::Lambda::new(
        lambda
            .arguments()
            .iter()
            .map(|argument| hir::Argument::new(argument.name(), argument.type_().clone()))
            .collect(),
        lambda.result_type().clone(),
        compile_block(lambda.body())?,
        lambda.position().clone(),
    ))
}

fn compile_block(block: &ast::Block) -> Result<hir::Block, CompileError> {
    Ok(hir::Block::new(
        block
            .statements()
            .iter()
            .map(|statement| compile_statement(statement))
            .collect::<Result<_, _>>()?,
        compile_expression(block.expression())?,
    ))
}

fn compile_statement(statement: &ast::Statement) -> Result<hir::Statement, CompileError> {
    Ok(hir::Statement::new(
        statement.name().map(String::from),
        compile_expression(statement.expression())?,
        None,
        statement.position().clone(),
    ))
}

fn compile_expression(expression: &ast::Expression) -> Result<hir::Expression, CompileError> {
    Ok(match expression {
        ast::Expression::BinaryOperation(operation) => {
            let lhs = compile_expression(operation.lhs())?;
            let rhs = compile_expression(operation.rhs())?;
            let position = operation.position().clone();

            match operation.operator() {
                ast::BinaryOperator::Add => {
                    hir::ArithmeticOperation::new(hir::ArithmeticOperator::Add, lhs, rhs, position)
                        .into()
                }
                ast::BinaryOperator::Subtract => hir::ArithmeticOperation::new(
                    hir::ArithmeticOperator::Subtract,
                    lhs,
                    rhs,
                    position,
                )
                .into(),
                ast::BinaryOperator::Multiply => hir::ArithmeticOperation::new(
                    hir::ArithmeticOperator::Multiply,
                    lhs,
                    rhs,
                    position,
                )
                .into(),
                ast::BinaryOperator::Divide => hir::ArithmeticOperation::new(
                    hir::ArithmeticOperator::Divide,
                    lhs,
                    rhs,
                    position,
                )
                .into(),

                ast::BinaryOperator::And => {
                    hir::BooleanOperation::new(hir::BooleanOperator::And, lhs, rhs, position).into()
                }
                ast::BinaryOperator::Or => {
                    hir::BooleanOperation::new(hir::BooleanOperator::Or, lhs, rhs, position).into()
                }

                ast::BinaryOperator::Equal => hir::EqualityOperation::new(
                    None,
                    hir::EqualityOperator::Equal,
                    lhs,
                    rhs,
                    position,
                )
                .into(),
                ast::BinaryOperator::NotEqual => hir::EqualityOperation::new(
                    None,
                    hir::EqualityOperator::NotEqual,
                    lhs,
                    rhs,
                    position,
                )
                .into(),

                ast::BinaryOperator::LessThan => {
                    hir::OrderOperation::new(hir::OrderOperator::LessThan, lhs, rhs, position)
                        .into()
                }
                ast::BinaryOperator::LessThanOrEqual => hir::OrderOperation::new(
                    hir::OrderOperator::LessThanOrEqual,
                    lhs,
                    rhs,
                    position,
                )
                .into(),
                ast::BinaryOperator::GreaterThan => {
                    hir::OrderOperation::new(hir::OrderOperator::GreaterThan, lhs, rhs, position)
                        .into()
                }
                ast::BinaryOperator::GreaterThanOrEqual => hir::OrderOperation::new(
                    hir::OrderOperator::GreaterThanOrEqual,
                    lhs,
                    rhs,
                    position,
                )
                .into(),
            }
        }
        ast::Expression::Boolean(boolean) => {
            hir::Boolean::new(boolean.value(), boolean.position().clone()).into()
        }
        ast::Expression::If(if_) => compile_if(if_.branches(), if_.else_(), if_.position())?.into(),
        ast::Expression::Lambda(lambda) => compile_lambda(lambda)?.into(),
        ast::Expression::Number(number) => {
            hir::Number::new(number.value(), number.position().clone()).into()
        }
        _ => todo!(),
    })
}

fn compile_if(
    branches: &[ast::IfBranch],
    else_: &ast::Block,
    position: &Position,
) -> Result<hir::If, CompileError> {
    Ok(match branches {
        [] => return Err(CompileError::TooFewBranchesInIf(position.clone())),
        [then] => hir::If::new(
            compile_expression(then.condition())?,
            compile_block(then.block())?,
            compile_block(else_)?,
            None,
            position.clone(),
        ),
        [then, ..] => hir::If::new(
            compile_expression(then.condition())?,
            compile_block(then.block())?,
            hir::Block::new(vec![], compile_if(&branches[1..], else_, position)?),
            None,
            position.clone(),
        ),
    })
}

fn is_record_open(elements: &[types::RecordElement]) -> bool {
    elements
        .iter()
        .all(|element| is_name_public(element.name()))
}

fn is_name_public(name: &str) -> bool {
    name.chars().next().unwrap().is_ascii_uppercase()
}
