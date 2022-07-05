use super::{AnalysisContext, AnalysisError};
use crate::{
    analysis::{record_field_resolver, type_canonicalizer, union_type_creator},
    ir::*,
    types::{self, Type},
};
use fnv::FnvHashMap;
use position::Position;

pub fn extract_from_expression(
    context: &AnalysisContext,
    expression: &Expression,
    variables: &FnvHashMap<String, Type>,
) -> Result<Type, AnalysisError> {
    let extract_from_expression =
        |expression, variables: &_| extract_from_expression(context, expression, variables);

    Ok(match expression {
        Expression::Boolean(boolean) => types::Boolean::new(boolean.position().clone()).into(),
        Expression::BuiltInCall(call) => extract_from_call_like(
            context,
            call.function_type(),
            call.position(),
            call.position(),
        )?,
        Expression::Call(call) => extract_from_call_like(
            context,
            call.function_type(),
            call.position(),
            call.function().position(),
        )?,
        Expression::If(if_) => types::Union::new(
            extract_from_expression(if_.then(), variables)?,
            extract_from_expression(if_.else_(), variables)?,
            if_.position().clone(),
        )
        .into(),
        Expression::IfList(if_) => {
            let list_type = type_canonicalizer::canonicalize_list(
                &extract_from_expression(if_.list(), variables)?,
                context.types(),
            )?
            .ok_or_else(|| AnalysisError::ListExpected(if_.list().position().clone()))?;

            types::Union::new(
                extract_from_expression(
                    if_.then(),
                    &variables
                        .clone()
                        .into_iter()
                        .chain([
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
        Expression::IfMap(if_) => {
            let map_type = type_canonicalizer::canonicalize_map(
                &extract_from_expression(if_.map(), variables)?,
                context.types(),
            )?
            .ok_or_else(|| AnalysisError::MapExpected(if_.map().position().clone()))?;

            types::Union::new(
                extract_from_expression(
                    if_.then(),
                    &variables
                        .clone()
                        .into_iter()
                        .chain([(if_.name().into(), map_type.value().clone())])
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
                            .chain([(if_.name().into(), branch.type_().clone())])
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
                                    .chain([(
                                        if_.name().into(),
                                        branch
                                            .type_()
                                            .ok_or_else(|| {
                                                AnalysisError::TypeNotInferred(
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
        Expression::ListComprehension(comprehension) => types::List::new(
            comprehension.output_type().clone(),
            comprehension.position().clone(),
        )
        .into(),
        Expression::Let(let_) => extract_from_expression(
            let_.expression(),
            &variables
                .clone()
                .into_iter()
                .chain(
                    let_.name()
                        .map(|name| -> Result<_, AnalysisError> {
                            Ok((
                                name.into(),
                                let_.type_()
                                    .ok_or_else(|| {
                                        AnalysisError::TypeNotInferred(let_.position().clone())
                                    })?
                                    .clone(),
                            ))
                        })
                        .transpose()?,
                )
                .collect(),
        )?,
        Expression::Map(map) => types::Map::new(
            map.key_type().clone(),
            map.value_type().clone(),
            map.position().clone(),
        )
        .into(),
        Expression::MapIterationComprehension(comprehension) => types::List::new(
            comprehension.element_type().clone(),
            comprehension.position().clone(),
        )
        .into(),
        Expression::None(none) => types::None::new(none.position().clone()).into(),
        Expression::Number(number) => types::Number::new(number.position().clone()).into(),
        Expression::Operation(operation) => match operation {
            Operation::Arithmetic(_) => types::Number::new(expression.position().clone()).into(),
            Operation::Boolean(_)
            | Operation::Equality(_)
            | Operation::Not(_)
            | Operation::Order(_) => types::Boolean::new(expression.position().clone()).into(),
            Operation::Try(operation) => operation
                .type_()
                .ok_or_else(|| AnalysisError::TypeNotInferred(operation.position().clone()))?
                .clone(),
        },
        Expression::RecordConstruction(construction) => construction.type_().clone(),
        Expression::RecordDeconstruction(deconstruction) => record_field_resolver::resolve(
            deconstruction
                .type_()
                .ok_or_else(|| AnalysisError::TypeNotInferred(deconstruction.position().clone()))?,
            deconstruction.position(),
            context.types(),
            context.records(),
        )?
        .iter()
        .find(|field| field.name() == deconstruction.field_name())
        .ok_or_else(|| AnalysisError::UnknownRecordField(deconstruction.position().clone()))?
        .type_()
        .clone(),
        Expression::RecordUpdate(update) => update.type_().clone(),
        Expression::String(string) => types::ByteString::new(string.position().clone()).into(),
        Expression::Thunk(thunk) => types::Function::new(
            vec![],
            thunk
                .type_()
                .ok_or_else(|| AnalysisError::TypeNotInferred(thunk.position().clone()))?
                .clone(),
            thunk.position().clone(),
        )
        .into(),
        Expression::TypeCoercion(coercion) => coercion.to().clone(),
        Expression::Variable(variable) => variables
            .get(variable.name())
            .cloned()
            .ok_or_else(|| AnalysisError::VariableNotFound(variable.clone()))?,
    })
}

fn extract_from_call_like(
    context: &AnalysisContext,
    type_: Option<&Type>,
    call_position: &Position,
    function_position: &Position,
) -> Result<Type, AnalysisError> {
    Ok(type_canonicalizer::canonicalize_function(
        type_.ok_or_else(|| AnalysisError::TypeNotInferred(call_position.clone()))?,
        context.types(),
    )?
    .ok_or_else(|| AnalysisError::FunctionExpected(function_position.clone()))?
    .result()
    .clone())
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

    fn empty_context() -> AnalysisContext {
        AnalysisContext::new(Default::default(), Default::default(), None)
    }

    #[test]
    fn extract_from_let() {
        assert_eq!(
            extract_from_expression(
                &empty_context(),
                &Let::new(
                    Some("x".into()),
                    Some(types::None::new(Position::fake()).into()),
                    None::new(Position::fake()),
                    Variable::new("x", Position::fake()),
                    Position::fake(),
                )
                .into(),
                &Default::default(),
            )
            .unwrap(),
            types::None::new(Position::fake()).into(),
        );
    }

    #[test]
    fn extract_from_try_operation() {
        assert_eq!(
            extract_from_expression(
                &empty_context(),
                &TryOperation::new(
                    Some(types::None::new(Position::fake()).into()),
                    Variable::new("x", Position::fake()),
                    Position::fake()
                )
                .into(),
                &Default::default(),
            )
            .unwrap(),
            types::None::new(Position::fake()).into(),
        );
    }

    #[test]
    fn extract_from_thunk() {
        assert_eq!(
            extract_from_expression(
                &empty_context(),
                &Thunk::new(
                    Some(types::None::new(Position::fake()).into()),
                    Variable::new("x", Position::fake()),
                    Position::fake()
                )
                .into(),
                &Default::default(),
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
                &empty_context(),
                &Thunk::new(None, Variable::new("x", Position::fake()), Position::fake()).into(),
                &Default::default(),
            ),
            Err(AnalysisError::TypeNotInferred(Position::fake())),
        );
    }

    mod built_in_call {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn extract_from_size() {
            assert_eq!(
                extract_from_expression(
                    &empty_context(),
                    &BuiltInCall::new(
                        Some(
                            types::Function::new(
                                vec![types::List::new(
                                    types::None::new(Position::fake()),
                                    Position::fake()
                                )
                                .into()],
                                types::Number::new(Position::fake()),
                                Position::fake()
                            )
                            .into()
                        ),
                        BuiltInFunction::Size,
                        vec![List::new(
                            types::None::new(Position::fake()),
                            vec![],
                            Position::fake()
                        )
                        .into()],
                        Position::fake()
                    )
                    .into(),
                    &Default::default(),
                ),
                Ok(types::Number::new(Position::fake()).into())
            );
        }

        #[test]
        fn extract_from_spawn() {
            let function_type =
                types::Function::new(vec![], types::None::new(Position::fake()), Position::fake());

            assert_eq!(
                extract_from_expression(
                    &empty_context(),
                    &BuiltInCall::new(
                        Some(
                            types::Function::new(
                                vec![function_type.clone().into()],
                                function_type.clone(),
                                Position::fake()
                            )
                            .into()
                        ),
                        BuiltInFunction::Spawn,
                        vec![Lambda::new(
                            vec![],
                            types::None::new(Position::fake()),
                            None::new(Position::fake()),
                            Position::fake()
                        )
                        .into()],
                        Position::fake()
                    )
                    .into(),
                    &Default::default(),
                ),
                Ok(function_type.into())
            );
        }
    }
}
