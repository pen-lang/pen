use super::{context::AnalysisContext, error::AnalysisError};
use crate::{analysis::type_subsumption_checker, ir::*, types, types::Type};

pub fn validate(context: &AnalysisContext, module: &Module) -> Result<(), AnalysisError> {
    for definition in module.function_definitions() {
        validate_lambda(context, definition.lambda())?;
    }

    Ok(())
}

fn validate_lambda(context: &AnalysisContext, lambda: &Lambda) -> Result<(), AnalysisError> {
    validate_expression(context, lambda.body(), Some(lambda.result_type()))
}

fn validate_expression(
    context: &AnalysisContext,
    expression: &Expression,
    result_type: Option<&Type>,
) -> Result<(), AnalysisError> {
    let validate = |expression| validate_expression(context, expression, result_type);

    match expression {
        Expression::Call(call) => {
            validate(call.function())?;

            for argument in call.arguments() {
                validate(argument)?;
            }
        }
        Expression::If(if_) => {
            validate(if_.condition())?;
            validate(if_.then())?;
            validate(if_.else_())?;
        }
        Expression::IfList(if_) => {
            validate(if_.list())?;
            validate(if_.then())?;
            validate(if_.else_())?;
        }
        Expression::IfMap(if_) => {
            validate(if_.map())?;
            validate(if_.key())?;
            validate(if_.then())?;
            validate(if_.else_())?;
        }
        Expression::IfType(if_) => {
            validate(if_.argument())?;

            for branch in if_.branches() {
                validate(branch.expression())?;
            }

            if let Some(branch) = if_.else_() {
                validate(branch.expression())?;
            }
        }
        Expression::TypeCoercion(coercion) => {
            validate(coercion.argument())?;
        }
        Expression::Lambda(lambda) => {
            validate_lambda(context, lambda)?;
        }
        Expression::Let(let_) => {
            validate(let_.bound_expression())?;
            validate(let_.expression())?;
        }
        Expression::List(list) => {
            for element in list.elements() {
                validate_expression(
                    context,
                    match element {
                        ListElement::Multiple(expression) => expression,
                        ListElement::Single(expression) => expression,
                    },
                    None,
                )?;
            }
        }
        Expression::ListComprehension(comprehension) => {
            for branch in comprehension.branches() {
                validate_expression(context, branch.iteratee(), None)?;
            }

            validate_expression(context, comprehension.element(), None)?;
        }
        Expression::Map(map) => {
            for element in map.elements() {
                match element {
                    MapElement::Insertion(entry) => {
                        validate(entry.key())?;
                        validate(entry.value())?
                    }
                    MapElement::Map(expression) => validate(expression)?,
                }
            }
        }
        Expression::Operation(operation) => match operation {
            Operation::Addition(operation) => {
                validate(operation.lhs())?;
                validate(operation.rhs())?;
            }
            Operation::Arithmetic(operation) => {
                validate(operation.lhs())?;
                validate(operation.rhs())?;
            }
            Operation::Boolean(operation) => {
                validate(operation.lhs())?;
                validate(operation.rhs())?;
            }
            Operation::Equality(operation) => {
                validate(operation.lhs())?;
                validate(operation.rhs())?;
            }
            Operation::Not(operation) => {
                validate(operation.expression())?;
            }
            Operation::Order(operation) => {
                validate(operation.lhs())?;
                validate(operation.rhs())?;
            }
            Operation::Try(operation) => {
                let position = operation.position();

                if let Some(result_type) = result_type {
                    if !type_subsumption_checker::check(
                        &types::Error::new(position.clone()).into(),
                        result_type,
                        context.types(),
                    )? {
                        return Err(AnalysisError::InvalidTryOperation(position.clone()));
                    }
                } else {
                    return Err(AnalysisError::TryOperationInList(position.clone()));
                }

                validate(operation.expression())?;
            }
        },
        Expression::RecordConstruction(construction) => {
            for field in construction.fields() {
                validate(field.expression())?;
            }
        }
        Expression::RecordDeconstruction(deconstruction) => {
            validate(deconstruction.record())?;
        }
        Expression::RecordUpdate(update) => {
            validate(update.record())?;

            for field in update.fields() {
                validate(field.expression())?;
            }
        }
        Expression::Thunk(thunk) => {
            validate_expression(
                context,
                thunk.expression(),
                Some(
                    thunk
                        .type_()
                        .ok_or_else(|| AnalysisError::TypeNotInferred(thunk.position().clone()))?,
                ),
            )?;
        }
        Expression::Boolean(_)
        | Expression::BuiltInFunction(_)
        | Expression::None(_)
        | Expression::Number(_)
        | Expression::String(_)
        | Expression::Variable(_) => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        analysis::type_collector,
        test::{FunctionDefinitionFake, ModuleFake, TypeDefinitionFake},
        types,
    };
    use position::{test::PositionFake, Position};

    const ERROR_TYPE_NAME: &str = "error";

    fn validate_module(module: &Module) -> Result<(), AnalysisError> {
        validate(
            &AnalysisContext::new(
                type_collector::collect(module),
                type_collector::collect_record_fields(module),
            ),
            module,
        )
    }

    #[test]
    fn validate_empty_module() -> Result<(), AnalysisError> {
        validate_module(&Module::empty())
    }

    #[test]
    fn fail_to_validate_lambda() {
        assert_eq!(
            validate_module(
                &Module::empty()
                    .set_type_definitions(vec![TypeDefinition::fake(
                        "error",
                        vec![],
                        false,
                        false,
                        false
                    )])
                    .set_function_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new(
                                "x",
                                types::Union::new(
                                    types::None::new(Position::fake()),
                                    types::Reference::new(ERROR_TYPE_NAME, Position::fake(),),
                                    Position::fake(),
                                ),
                            )],
                            types::None::new(Position::fake()),
                            TryOperation::new(
                                None,
                                Variable::new("x", Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
            ),
            Err(AnalysisError::InvalidTryOperation(Position::fake()))
        );
    }

    #[test]
    fn validate_thunk() {
        let union_type = types::Union::new(
            types::None::new(Position::fake()),
            types::Error::new(Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            validate_module(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", union_type.clone())],
                        types::Function::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Position::fake()
                        ),
                        Thunk::new(
                            Some(union_type.into()),
                            TryOperation::new(
                                None,
                                Variable::new("x", Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )
            ])),
            Ok(())
        );
    }

    #[test]
    fn fail_to_validate_thunk() {
        assert_eq!(
            validate_module(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new(
                            "x",
                            types::Union::new(
                                types::None::new(Position::fake()),
                                types::Error::new(Position::fake()),
                                Position::fake(),
                            ),
                        )],
                        types::Function::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Position::fake()
                        ),
                        Thunk::new(
                            Some(types::None::new(Position::fake()).into()),
                            TryOperation::new(
                                None,
                                Variable::new("x", Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )
            ])),
            Err(AnalysisError::InvalidTryOperation(Position::fake()))
        );
    }

    #[test]
    fn fail_to_validate_list() {
        assert_eq!(
            validate_module(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new(
                            "x",
                            types::Union::new(
                                types::None::new(Position::fake()),
                                types::Error::new(Position::fake()),
                                Position::fake(),
                            ),
                        )],
                        types::Function::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Position::fake()
                        ),
                        List::new(
                            types::None::new(Position::fake()),
                            vec![ListElement::Single(
                                TryOperation::new(
                                    None,
                                    Variable::new("x", Position::fake()),
                                    Position::fake(),
                                )
                                .into()
                            )],
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )
            ])),
            Err(AnalysisError::TryOperationInList(Position::fake()))
        );
    }

    #[test]
    fn fail_to_validate_element_of_list_comprehension() {
        assert_eq!(
            validate_module(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::Function::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Position::fake()
                        ),
                        ListComprehension::new(
                            types::None::new(Position::fake()),
                            TryOperation::new(
                                None,
                                Variable::new("x", Position::fake()),
                                Position::fake(),
                            ),
                            vec![ListComprehensionBranch::new(
                                None,
                                "x",
                                None,
                                Variable::new("xs", Position::fake()),
                                Position::fake(),
                            )],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )
            ])),
            Err(AnalysisError::TryOperationInList(Position::fake()))
        );
    }

    #[test]
    fn fail_to_validate_list_of_list_comprehension() {
        assert_eq!(
            validate_module(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::Function::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Position::fake()
                        ),
                        ListComprehension::new(
                            types::None::new(Position::fake()),
                            None::new(Position::fake()),
                            vec![ListComprehensionBranch::new(
                                None,
                                "x",
                                None,
                                TryOperation::new(
                                    None,
                                    Variable::new("xs", Position::fake()),
                                    Position::fake(),
                                ),
                                Position::fake()
                            )],
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )
            ])),
            Err(AnalysisError::TryOperationInList(Position::fake()))
        );
    }
}
