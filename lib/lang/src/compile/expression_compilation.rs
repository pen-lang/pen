use super::{type_compilation, type_context::TypeContext, CompileError};
use crate::{compile::type_compilation::NONE_RECORD_TYPE_NAME, hir::*};

const CLOSURE_NAME: &str = "$closure";

pub fn compile(
    expression: &Expression,
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
                compile_block(lambda.body(), type_context)?,
                type_compilation::compile(lambda.result_type(), type_context)?,
            ),
            mir::ir::Variable::new(CLOSURE_NAME),
        )
        .into(),
        Expression::None(_) => {
            mir::ir::Record::new(mir::types::Record::new(NONE_RECORD_TYPE_NAME), vec![]).into()
        }
        Expression::Number(number) => mir::ir::Expression::Number(number.value()),
        Expression::String(string) => mir::ir::ByteString::new(string.value()).into(),
        Expression::Variable(variable) => mir::ir::Variable::new(variable.name()).into(),
        _ => todo!(),
    })
}

pub fn compile_block(
    block: &Block,
    type_context: &TypeContext,
) -> Result<mir::ir::Expression, CompileError> {
    let mut expression = compile(block.expression(), type_context)?;

    for assignment in block.assignments().iter().rev() {
        expression = mir::ir::Let::new(
            assignment.name(),
            type_compilation::compile(
                assignment
                    .type_()
                    .ok_or_else(|| CompileError::TypeNotInferred(assignment.position().clone()))?,
                type_context,
            )?,
            compile(assignment.expression(), type_context)?,
            expression,
        )
        .into();
    }

    Ok(expression)
}
