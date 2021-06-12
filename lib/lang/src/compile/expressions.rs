use super::{type_compilation, type_context::TypeContext, CompileError};
use crate::hir::{Block, Expression};
use std::collections::HashMap;

const CLOSURE_NAME: &str = "$closure";

pub fn compile(
    expression: &Expression,
    variables: &HashMap<String, mir::ir::Expression>,
    type_context: &TypeContext,
) -> Result<mir::ir::Expression, CompileError> {
    Ok(match expression {
        Expression::Boolean(boolean) => mir::ir::Expression::Boolean(boolean.value()),
        Expression::Lambda(lambda) => mir::ir::LetRecursive::new(
            mir::ir::Definition::new(
                CLOSURE_NAME,
                lambda
                    .arguments()
                    .iter()
                    .map(|argument| -> Result<_, CompileError> {
                        Ok(mir::ir::Argument::new(
                            argument.name(),
                            type_compilation::compile(argument.type_(), type_context)?,
                        ))
                    })
                    .collect::<Result<_, _>>()?,
                compile_block(lambda.body(), variables, type_context)?,
                type_compilation::compile(lambda.type_(), type_context)?,
            ),
            mir::ir::Variable::new(CLOSURE_NAME),
        )
        .into(),
        Expression::Number(number) => mir::ir::Expression::Number(number.value()),
        Expression::Variable(variable) => variables
            .get(variable.name())
            .cloned()
            .unwrap_or_else(|| mir::ir::Variable::new(variable.name()).into()),
        _ => todo!(),
    })
}

pub fn compile_block(
    block: &Block,
    variables: &HashMap<String, mir::ir::Expression>,
    type_context: &TypeContext,
) -> Result<mir::ir::Expression, CompileError> {
    let mut expression = compile(block.expression(), variables, type_context)?;

    for assignment in block.assignments().iter().rev() {
        expression = mir::ir::Let::new(
            assignment.name(),
            type_compilation::compile(assignment.type_(), type_context)?,
            compile(assignment.expression(), variables, type_context)?,
            expression,
        )
        .into();
    }

    Ok(expression)
}
