use super::{context::CompileContext, CompileError};
use hir::{
    analysis::types::type_subsumption_checker,
    ir::*,
    types::{self, Type},
};

pub fn validate(module: &Module, context: &CompileContext) -> Result<(), CompileError> {
    for definition in module.definitions() {
        validate_lambda(definition.lambda(), context)?;
    }

    Ok(())
}

fn validate_lambda(lambda: &Lambda, context: &CompileContext) -> Result<(), CompileError> {
    validate_expression(lambda.body(), Some(lambda.result_type()), context)
}

fn validate_expression(
    expression: &Expression,
    result_type: Option<&Type>,
    context: &CompileContext,
) -> Result<(), CompileError> {
    let validate = |expression| validate_expression(expression, result_type, context);

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
            validate(if_.argument())?;
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
            validate_lambda(lambda, context)?;
        }
        Expression::Let(let_) => {
            validate(let_.bound_expression())?;
            validate(let_.expression())?;
        }
        Expression::List(list) => {
            for element in list.elements() {
                validate_expression(
                    match element {
                        ListElement::Multiple(expression) => expression,
                        ListElement::Single(expression) => expression,
                    },
                    None,
                    context,
                )?;
            }
        }
        Expression::ListComprehension(comprehension) => {
            validate_expression(comprehension.element(), None, context)?;
            validate_expression(comprehension.list(), None, context)?;
        }
        Expression::Operation(operation) => match operation {
            Operation::Arithmetic(operation) => {
                validate(operation.lhs())?;
                validate(operation.rhs())?;
            }
            Operation::Spawn(operation) => {
                validate_lambda(operation.function(), context)?;
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
                if let Some(result_type) = result_type {
                    if !type_subsumption_checker::check(
                        &types::Reference::new(
                            &context.configuration()?.error_type.error_type_name,
                            result_type.position().clone(),
                        )
                        .into(),
                        result_type,
                        context.types(),
                    )? {
                        return Err(CompileError::InvalidTryOperation(
                            operation.position().clone(),
                        ));
                    }
                } else {
                    return Err(CompileError::TryOperationInList(
                        operation.position().clone(),
                    ));
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
                thunk.expression(),
                Some(
                    thunk
                        .type_()
                        .ok_or_else(|| CompileError::TypeNotInferred(thunk.position().clone()))?,
                ),
                context,
            )?;
        }
        Expression::Boolean(_)
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
        compile_configuration::COMPILE_CONFIGURATION,
        error_type_configuration::ERROR_TYPE_CONFIGURATION,
    };
    use hir::test::{DefinitionFake, ModuleFake, TypeDefinitionFake};
    use position::{test::PositionFake, Position};

    fn validate_module(module: &Module) -> Result<(), CompileError> {
        validate(
            module,
            &CompileContext::new(module, COMPILE_CONFIGURATION.clone().into()),
        )
    }

    #[test]
    fn validate_empty_module() -> Result<(), CompileError> {
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
                    .set_definitions(vec![Definition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new(
                                "x",
                                types::Union::new(
                                    types::None::new(Position::fake()),
                                    types::Reference::new(
                                        &ERROR_TYPE_CONFIGURATION.error_type_name,
                                        Position::fake(),
                                    ),
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
            Err(CompileError::InvalidTryOperation(Position::fake()))
        );
    }

    #[test]
    fn validate_thunk() {
        let error_type =
            types::Reference::new(&ERROR_TYPE_CONFIGURATION.error_type_name, Position::fake());
        let union_type = types::Union::new(
            types::None::new(Position::fake()),
            error_type,
            Position::fake(),
        );

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
                    .set_definitions(vec![Definition::fake(
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
                    )])
            ),
            Ok(())
        );
    }

    #[test]
    fn fail_to_validate_thunk() {
        let error_type =
            types::Reference::new(&ERROR_TYPE_CONFIGURATION.error_type_name, Position::fake());

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
                    .set_definitions(vec![Definition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new(
                                "x",
                                types::Union::new(
                                    types::None::new(Position::fake()),
                                    error_type,
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
                    )])
            ),
            Err(CompileError::InvalidTryOperation(Position::fake()))
        );
    }

    #[test]
    fn fail_to_validate_list() {
        let error_type =
            types::Reference::new(&ERROR_TYPE_CONFIGURATION.error_type_name, Position::fake());

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
                    .set_definitions(vec![Definition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new(
                                "x",
                                types::Union::new(
                                    types::None::new(Position::fake()),
                                    error_type,
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
                    )])
            ),
            Err(CompileError::TryOperationInList(Position::fake()))
        );
    }

    #[test]
    fn fail_to_validate_element_of_list_comprehension() {
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
                    .set_definitions(vec![Definition::fake(
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
                                "x",
                                Variable::new("xs", Position::fake()),
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
            ),
            Err(CompileError::TryOperationInList(Position::fake()))
        );
    }

    #[test]
    fn fail_to_validate_list_of_list_comprehension() {
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
                    .set_definitions(vec![Definition::fake(
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
                                "x",
                                TryOperation::new(
                                    None,
                                    Variable::new("xs", Position::fake()),
                                    Position::fake(),
                                ),
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
            ),
            Err(CompileError::TryOperationInList(Position::fake()))
        );
    }
}
