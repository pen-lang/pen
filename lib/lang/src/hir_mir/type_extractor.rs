use super::{type_context::TypeContext, CompileError};
use crate::{
    hir::*,
    types::{
        self,
        analysis::{type_resolver, union_type_creator},
        Type,
    },
};
use std::collections::HashMap;

pub fn extract_from_expression(
    expression: &Expression,
    variables: &HashMap<String, Type>,
    type_context: &TypeContext,
) -> Result<Type, CompileError> {
    let extract_from_expression =
        |expression, variables: &_| extract_from_expression(expression, variables, type_context);

    Ok(match expression {
        Expression::Boolean(boolean) => types::Boolean::new(boolean.position().clone()).into(),
        Expression::Call(call) => type_resolver::resolve_to_function(
            call.function_type()
                .ok_or_else(|| CompileError::TypeNotInferred(call.position().clone()))?,
            type_context.types(),
        )?
        .ok_or_else(|| CompileError::FunctionExpected(call.function().position().clone()))?
        .result()
        .clone(),
        Expression::If(if_) => types::Union::new(
            extract_from_expression(if_.then(), variables)?,
            extract_from_expression(if_.else_(), variables)?,
            if_.position().clone(),
        )
        .into(),
        Expression::IfList(if_) => {
            let list_type = type_resolver::resolve_to_list(
                &extract_from_expression(if_.argument(), variables)?,
                type_context.types(),
            )?
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
                )?,
                extract_from_expression(if_.else_(), variables)?,
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
                                        extract_from_expression(if_.argument(), variables)?,
                                    )])
                                    .collect(),
                            )
                        })
                        .transpose()?,
                )
                .collect::<Vec<_>>(),
            if_.position(),
        )
        .unwrap(),
        Expression::Lambda(lambda) => extract_from_lambda(lambda).into(),
        Expression::List(list) => {
            types::List::new(list.type_().clone(), list.position().clone()).into()
        }
        Expression::Let(let_) => extract_from_expression(
            let_.expression(),
            &variables
                .clone()
                .into_iter()
                .chain(
                    let_.name()
                        .map(|name| -> Result<_, CompileError> {
                            Ok((
                                name.into(),
                                let_.type_()
                                    .ok_or_else(|| {
                                        CompileError::TypeNotInferred(let_.position().clone())
                                    })?
                                    .clone(),
                            ))
                        })
                        .transpose()?,
                )
                .collect(),
        )?,
        Expression::None(none) => types::None::new(none.position().clone()).into(),
        Expression::Number(number) => types::Number::new(number.position().clone()).into(),
        Expression::Operation(operation) => match operation {
            Operation::Arithmetic(_) => types::Number::new(expression.position().clone()).into(),
            Operation::Boolean(_)
            | Operation::Equality(_)
            | Operation::Not(_)
            | Operation::Order(_) => types::Boolean::new(expression.position().clone()).into(),
            Operation::Try(_) => todo!(),
        },
        Expression::RecordConstruction(construction) => construction.type_().clone(),
        Expression::RecordDeconstruction(element) => type_resolver::resolve_record_elements(
            element
                .type_()
                .ok_or_else(|| CompileError::TypeNotInferred(element.position().clone()))?,
            element.position(),
            type_context.types(),
            type_context.records(),
        )?
        .get(element.element_name())
        .ok_or_else(|| CompileError::RecordDeconstructionUnknown(element.position().clone()))?
        .clone(),
        Expression::RecordUpdate(update) => update.type_().clone(),
        Expression::String(string) => types::ByteString::new(string.position().clone()).into(),
        Expression::TypeCoercion(coercion) => coercion.to().clone(),
        Expression::Variable(variable) => variables
            .get(variable.name())
            .cloned()
            .ok_or_else(|| CompileError::VariableNotFound(variable.clone()))?,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::Position;
    use pretty_assertions::assert_eq;

    #[test]
    fn extract_from_let() {
        assert_eq!(
            extract_from_expression(
                &Let::new(
                    Some("x".into()),
                    Some(types::None::new(Position::dummy()).into()),
                    None::new(Position::dummy()),
                    Variable::new("x", Position::dummy()),
                    Position::dummy(),
                )
                .into(),
                &Default::default(),
                &TypeContext::dummy(Default::default(), Default::default()),
            )
            .unwrap(),
            types::None::new(Position::dummy()).into(),
        );
    }
}
