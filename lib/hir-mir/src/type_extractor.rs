use super::{type_context::TypeContext, CompileError};
use hir::{
    analysis::types::{record_field_resolver, type_canonicalizer, union_type_creator},
    ir::*,
    types::{self, Type},
};
use std::collections::BTreeMap;

pub fn extract_from_expression(
    expression: &Expression,
    variables: &BTreeMap<String, Type>,
    type_context: &TypeContext,
) -> Result<Type, CompileError> {
    let extract_from_expression =
        |expression, variables: &_| extract_from_expression(expression, variables, type_context);

    Ok(match expression {
        Expression::Boolean(boolean) => types::Boolean::new(boolean.position().clone()).into(),
        Expression::Call(call) => type_canonicalizer::canonicalize_function(
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
            let list_type = type_canonicalizer::canonicalize_list(
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
                            (
                                if_.first_name().into(),
                                types::Function::new(
                                    vec![],
                                    list_type.element().clone(),
                                    if_.position().clone(),
                                )
                                .into(),
                            ),
                            (if_.rest_name().into(), list_type.clone().into()),
                        ])
                        .collect(),
                )?,
                extract_from_expression(if_.else_(), variables)?,
                if_.position().clone(),
            )
            .into()
        }
        Expression::IfType(if_) => union_type_creator::create(
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
                        .map(|branch| {
                            extract_from_expression(
                                branch.expression(),
                                &variables
                                    .clone()
                                    .into_iter()
                                    .chain(vec![(
                                        if_.name().into(),
                                        branch
                                            .type_()
                                            .ok_or_else(|| {
                                                CompileError::TypeNotInferred(
                                                    branch.position().clone(),
                                                )
                                            })?
                                            .clone(),
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
            Operation::Spawn(operation) => types::Function::new(
                vec![],
                operation.function().result_type().clone(),
                operation.position().clone(),
            )
            .into(),
            Operation::Boolean(_)
            | Operation::Equality(_)
            | Operation::Not(_)
            | Operation::Order(_) => types::Boolean::new(expression.position().clone()).into(),
            Operation::Try(operation) => operation
                .type_()
                .ok_or_else(|| CompileError::TypeNotInferred(operation.position().clone()))?
                .clone(),
        },
        Expression::RecordConstruction(construction) => construction.type_().clone(),
        Expression::RecordDeconstruction(deconstruction) => record_field_resolver::resolve(
            deconstruction
                .type_()
                .ok_or_else(|| CompileError::TypeNotInferred(deconstruction.position().clone()))?,
            deconstruction.position(),
            type_context.types(),
            type_context.records(),
        )?
        .iter()
        .find(|field| field.name() == deconstruction.field_name())
        .ok_or_else(|| CompileError::RecordFieldUnknown(deconstruction.position().clone()))?
        .type_()
        .clone(),
        Expression::RecordUpdate(update) => update.type_().clone(),
        Expression::String(string) => types::ByteString::new(string.position().clone()).into(),
        Expression::Thunk(thunk) => types::Function::new(
            vec![],
            thunk
                .type_()
                .ok_or_else(|| CompileError::TypeNotInferred(thunk.position().clone()))?
                .clone(),
            thunk.position().clone(),
        )
        .into(),
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
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    #[test]
    fn extract_from_let() {
        assert_eq!(
            extract_from_expression(
                &Let::new(
                    Some("x".into()),
                    Some(types::None::new(Position::fake()).into()),
                    None::new(Position::fake()),
                    Variable::new("x", Position::fake()),
                    Position::fake(),
                )
                .into(),
                &Default::default(),
                &TypeContext::dummy(Default::default(), Default::default()),
            )
            .unwrap(),
            types::None::new(Position::fake()).into(),
        );
    }

    #[test]
    fn extract_from_try_operation() {
        assert_eq!(
            extract_from_expression(
                &TryOperation::new(
                    Some(types::None::new(Position::fake()).into()),
                    Variable::new("x", Position::fake()),
                    Position::fake()
                )
                .into(),
                &Default::default(),
                &TypeContext::dummy(Default::default(), Default::default()),
            )
            .unwrap(),
            types::None::new(Position::fake()).into(),
        );
    }

    #[test]
    fn extract_from_thunk() {
        assert_eq!(
            extract_from_expression(
                &Thunk::new(
                    Some(types::None::new(Position::fake()).into()),
                    Variable::new("x", Position::fake()),
                    Position::fake()
                )
                .into(),
                &Default::default(),
                &TypeContext::dummy(Default::default(), Default::default()),
            ),
            Ok(
                types::Function::new(vec![], types::None::new(Position::fake()), Position::fake())
                    .into()
            )
        );
    }

    #[test]
    fn fail_to_extract_from_thunk() {
        assert_eq!(
            extract_from_expression(
                &Thunk::new(None, Variable::new("x", Position::fake()), Position::fake()).into(),
                &Default::default(),
                &TypeContext::dummy(Default::default(), Default::default()),
            ),
            Err(CompileError::TypeNotInferred(Position::fake())),
        );
    }

    #[test]
    fn extract_from_spawn_operation() {
        assert_eq!(
            extract_from_expression(
                &SpawnOperation::new(
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        None::new(Position::fake()),
                        Position::fake()
                    ),
                    Position::fake()
                )
                .into(),
                &Default::default(),
                &TypeContext::dummy(Default::default(), Default::default()),
            ),
            Ok(
                types::Function::new(vec![], types::None::new(Position::fake()), Position::fake())
                    .into()
            )
        );
    }
}
