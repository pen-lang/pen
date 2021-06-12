use super::CompileError;
use crate::{
    compile::type_resolution,
    hir::*,
    types::{self, Type},
};
use std::collections::HashMap;

pub fn extract_from_expression(
    expression: &Expression,
    types: &HashMap<String, Type>,
) -> Result<Type, CompileError> {
    Ok(match expression {
        Expression::Boolean(boolean) => types::Boolean::new(boolean.position().clone()).into(),
        Expression::Call(call) => type_resolution::resolve_to_function(
            call.function_type()
                .ok_or_else(|| CompileError::TypeNotInferred(call.position().clone()))?,
            types,
        )?
        .ok_or_else(|| CompileError::FunctionExpected(call.function().position().clone()))?
        .result()
        .clone(),
        Expression::If(if_) => if_
            .result_type()
            .ok_or_else(|| CompileError::TypeNotInferred(if_.position().clone()))?
            .clone(),
        Expression::IfList(if_) => if_
            .result_type()
            .ok_or_else(|| CompileError::TypeNotInferred(if_.position().clone()))?
            .clone(),
        Expression::IfType(if_) => if_
            .result_type()
            .ok_or_else(|| CompileError::TypeNotInferred(if_.position().clone()))?
            .clone(),
        Expression::None(none) => types::None::new(none.position().clone()).into(),
        Expression::Number(number) => types::Number::new(number.position().clone()).into(),
        Expression::String(string) => types::ByteString::new(string.position().clone()).into(),
        _ => todo!(),
    })
}

pub fn extract_from_block(
    block: &Block,
    types: &HashMap<String, Type>,
) -> Result<Type, CompileError> {
    extract_from_expression(block.expression(), types)
}
