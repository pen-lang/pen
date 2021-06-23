use super::CompileError;
use crate::{
    hir::*,
    hir_mir::union_type_creator,
    types::{self, Type},
};
use std::collections::HashMap;

pub fn extract_from_expression(
    expression: &Expression,
    variables: &HashMap<String, Type>,
    types: &HashMap<String, Type>,
) -> Result<Type, CompileError> {
    Ok(match expression {
        Expression::Boolean(boolean) => types::Boolean::new(boolean.position().clone()).into(),
        Expression::Call(call) => types::analysis::resolve_to_function(
            call.function_type()
                .ok_or_else(|| CompileError::TypeNotInferred(call.position().clone()))?,
            types,
        )?
        .ok_or_else(|| CompileError::FunctionExpected(call.function().position().clone()))?
        .result()
        .clone(),
        Expression::If(if_) => types::Union::new(
            extract_from_expression(if_.then(), variables, types)?,
            extract_from_expression(if_.else_(), variables, types)?,
            if_.position().clone(),
        )
        .into(),
        Expression::IfList(if_) => {
            let list_type = extract_from_expression(if_.argument(), variables, types)?
                .into_list()
                .ok_or_else(|| CompileError::ListExpected(if_.argument().position().clone()))?;

            types::Union::new(
                extract_from_expression(
                    if_.then(),
                    &variables
                        .clone()
                        .into_iter()
                        .chain(vec![
                            (if_.first_name().into(), list_type.element().clone()),
                            (if_.rest_name().into(), list_type.clone().into()),
                        ])
                        .collect(),
                    types,
                )?,
                extract_from_expression(if_.else_(), variables, types)?,
                if_.position().clone(),
            )
            .into()
        }
        Expression::IfType(if_) => union_type_creator::create_union_type(
            &if_.branches()
                .iter()
                .map(|branch| {
                    extract_from_expression(
                        branch.expression(),
                        &variables
                            .clone()
                            .into_iter()
                            .chain(vec![(if_.name().into(), branch.type_().clone())])
                            .collect(),
                        types,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .chain(
                    if_.else_()
                        .map(|expression| {
                            extract_from_expression(
                                expression,
                                &variables
                                    .clone()
                                    .into_iter()
                                    .chain(vec![(
                                        if_.name().into(),
                                        extract_from_expression(if_.argument(), variables, types)?,
                                    )])
                                    .collect(),
                                types,
                            )
                        })
                        .transpose()?,
                )
                .collect::<Vec<_>>(),
            if_.position(),
        )
        .unwrap(),
        Expression::Lambda(lambda) => extract_from_lambda(lambda).into(),
        Expression::None(none) => types::None::new(none.position().clone()).into(),
        Expression::Number(number) => types::Number::new(number.position().clone()).into(),
        Expression::String(string) => types::ByteString::new(string.position().clone()).into(),
        Expression::Variable(variable) => variables
            .get(variable.name())
            .cloned()
            .ok_or_else(|| CompileError::VariableNotFound(variable.clone()))?,
        _ => todo!(),
    })
}

pub fn extract_from_lambda(lambda: &Lambda) -> types::Function {
    types::Function::new(
        lambda
            .arguments()
            .iter()
            .map(|argument| argument.type_().clone())
            .collect(),
        lambda.result_type().clone(),
        lambda.position().clone(),
    )
}
