use super::type_context::TypeContext;
use super::CompileError;
use crate::compile::type_resolution;
use crate::{
    hir::*,
    types::{self, Type},
};

pub fn extract_from_expression(
    expression: &Expression,
    type_context: &TypeContext,
) -> Result<Type, CompileError> {
    Ok(match expression {
        Expression::Boolean(boolean) => types::Boolean::new(boolean.position().clone()).into(),
        Expression::Call(call) => type_resolution::resolve_to_function(
            call.function_type()
                .ok_or_else(|| CompileError::TypeNotInferred(call.position().clone().into()))?,
            type_context,
        )?
        .ok_or_else(|| CompileError::FunctionExpected(call.function().position().clone()))?
        .result()
        .clone(),
        Expression::If(if_) => if_
            .result_type()
            .ok_or_else(|| CompileError::TypeNotInferred(if_.position().clone().into()))?
            .clone(),
        Expression::IfList(if_) => if_
            .result_type()
            .ok_or_else(|| CompileError::TypeNotInferred(if_.position().clone().into()))?
            .clone(),
        Expression::IfType(if_) => if_
            .result_type()
            .ok_or_else(|| CompileError::TypeNotInferred(if_.position().clone().into()))?
            .clone(),
        Expression::Number(number) => types::Number::new(number.position().clone()).into(),
        _ => todo!(),
    })
}

pub fn extract_from_block(block: &Block, type_context: &TypeContext) -> Result<Type, CompileError> {
    extract_from_expression(block.expression(), type_context)
}
