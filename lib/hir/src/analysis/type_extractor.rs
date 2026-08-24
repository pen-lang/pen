use super::{AnalysisContext, AnalysisError};
use crate::{
    analysis::{record_field_resolver, type_canonicalizer, union_type_creator},
    ir::*,
    types::{self, Type},
};

pub fn extract_from_expression(
    context: &AnalysisContext,
    expression: &Expression,
    variables: &plist::FlailMap<String, Type>,
) -> Result<Type, AnalysisError> {
    let extract_from_expression =
        |expression, variables: &_| extract_from_expression(context, expression, variables);

    Ok(match expression {
        Expression::Boolean(boolean) => types::Boolean::new(boolean.position().clone()).into(),
        Expression::BuiltInFunction(function) => {
            return Err(AnalysisError::BuiltInFunctionNotCalled(
                function.position().clone(),
            ));
        }
        Expression::Call(call) => {
            let type_ = call
                .function_type()
                .ok_or_else(|| AnalysisError::TypeNotInferred(call.position().clone()))?;

            type_canonicalizer::canonicalize_function(type_, context.types())?
                .ok_or_else(|| {
                    AnalysisError::FunctionExpected(
                        call.function().position().clone(),
                        type_.clone(),
                    )
                })?
                .result()
                .clone()
        }
        Expression::If(if_) => types::Union::new(
            extract_from_expression(if_.then(), variables)?,
            extract_from_expression(if_.else_(), variables)?,
            if_.position().clone(),
        )
        .into(),
        Expression::IfList(if_) => {
            let type_ = extract_from_expression(if_.list(), variables)?;
            let list_type = type_canonicalizer::canonicalize_list(&type_, context.types())?
                .ok_or_else(|| AnalysisError::ListExpected(if_.list().position().clone(), type_))?;

            types::Union::new(
                extract_from_expression(
                    if_.then(),
                    &variables.insert_iter([
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
                    ]),
                )?,
                extract_from_expression(if_.else_(), variables)?,
                if_.position().clone(),
            )
            .into()
        }
        Expression::IfMap(if_) => {
            let type_ = extract_from_expression(if_.map(), variables)?;
            let map_type = type_canonicalizer::canonicalize_map(&type_, context.types())?
                .ok_or_else(|| AnalysisError::MapExpected(if_.map().position().clone(), type_))?;

            types::Union::new(
                extract_from_expression(
                    if_.then(),
                    &variables.insert(if_.name().into(), map_type.value().clone()),
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
                        &variables.insert(if_.name().into(), branch.type_().clone()),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .chain(
                    if_.else_()
                        .map(|branch| {
                            extract_from_expression(
                                branch.expression(),
                                &variables.insert(
                                    if_.name().into(),
                                    branch
                                        .type_()
                                        .ok_or_else(|| {
                                            AnalysisError::TypeNotInferred(
                                                branch.position().clone(),
                                            )
                                        })?
                                        .clone(),
                                ),
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
            comprehension.type_().clone(),
            comprehension.position().clone(),
        )
        .into(),
        Expression::Let(let_) => extract_from_expression(
            let_.expression(),
            &variables.insert_iter(
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
            ),
        )?,
        Expression::Map(map) => types::Map::new(
            map.key_type().clone(),
            map.value_type().clone(),
            map.position().clone(),
        )
        .into(),
        Expression::None(none) => types::None::new(none.position().clone()).into(),
        Expression::Number(number) => types::Number::new(number.position().clone()).into(),
        Expression::Operation(operation) => match operation {
            Operation::Addition(operation) => operation
                .type_()
                .ok_or_else(|| AnalysisError::TypeNotInferred(operation.position().clone()))?
                .clone(),
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
        Expression::RecordDeconstruction(deconstruction) => {
            let type_ = deconstruction
                .type_()
                .ok_or_else(|| AnalysisError::TypeNotInferred(deconstruction.position().clone()))?;

            record_field_resolver::resolve(
                type_,
                deconstruction.position(),
                context.types(),
                context.records(),
            )?
            .iter()
            .find(|field| field.name() == deconstruction.field_name())
            // TODO Use a field position.
            .ok_or_else(|| AnalysisError::UnknownRecordField(deconstruction.position().clone()))?
            .type_()
            .clone()
        }
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
    use position::{Position, test::PositionFake};
    use pretty_assertions::assert_eq;

    fn empty_context() -> AnalysisContext {
        AnalysisContext::new(Default::default(), Default::default())
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
                    &Call::new(
                        Some(
                            types::Function::new(
                                vec![
                                    types::List::new(
                                        types::None::new(Position::fake()),
                                        Position::fake()
                                    )
                                    .into()
                                ],
                                types::Number::new(Position::fake()),
                                Position::fake()
                            )
                            .into()
                        ),
                        BuiltInFunction::new(BuiltInFunctionName::Size, Position::fake()),
                        vec![
                            List::new(types::None::new(Position::fake()), vec![], Position::fake())
                                .into()
                        ],
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
                    &Call::new(
                        Some(
                            types::Function::new(
                                vec![function_type.clone().into()],
                                function_type.clone(),
                                Position::fake()
                            )
                            .into()
                        ),
                        BuiltInFunction::new(BuiltInFunctionName::Spawn, Position::fake()),
                        vec![
                            Lambda::new(
                                vec![],
                                types::None::new(Position::fake()),
                                None::new(Position::fake()),
                                Position::fake()
                            )
                            .into()
                        ],
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
