use super::{type_context::TypeContext, CompileError};
use crate::{
    hir::*,
    types::{self, analysis::type_subsumption_checker, Type},
};

pub fn validate(module: &Module, type_context: &TypeContext) -> Result<(), CompileError> {
    for definition in module.definitions() {
        validate_lambda(definition.lambda(), type_context)?;
    }

    Ok(())
}

fn validate_lambda(lambda: &Lambda, type_context: &TypeContext) -> Result<(), CompileError> {
    validate_expression(lambda.body(), lambda.result_type(), type_context)
}

fn validate_expression(
    expression: &Expression,
    result_type: &Type,
    type_context: &TypeContext,
) -> Result<(), CompileError> {
    let validate = |expression| validate_expression(expression, result_type, type_context);

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
            validate_lambda(lambda, type_context)?;
        }
        Expression::Let(let_) => {
            validate(let_.bound_expression())?;
            validate(let_.expression())?;
        }
        Expression::List(list) => {
            for element in list.elements() {
                validate(match element {
                    ListElement::Multiple(expression) => expression,
                    ListElement::Single(expression) => expression,
                })?;
            }
        }
        Expression::Operation(operation) => match operation {
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
                if !type_subsumption_checker::check(
                    &types::Reference::new(
                        &type_context.error_type_configuration().error_type_name,
                        result_type.position().clone(),
                    )
                    .into(),
                    result_type,
                    type_context.types(),
                )? {
                    return Err(CompileError::InvalidTryOperation(
                        operation.position().clone(),
                    ));
                }

                validate(operation.expression())?;
            }
        },
        Expression::RecordConstruction(construction) => {
            for element in construction.elements() {
                validate(element.expression())?;
            }
        }
        Expression::RecordDeconstruction(deconstruction) => {
            validate(deconstruction.record())?;
        }
        Expression::RecordUpdate(update) => {
            validate(update.record())?;

            for element in update.elements() {
                validate(element.expression())?;
            }
        }
        Expression::Thunk(thunk) => {
            validate_expression(
                thunk.expression(),
                thunk
                    .type_()
                    .ok_or_else(|| CompileError::TypeNotInferred(thunk.position().clone()))?,
                type_context,
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
    use super::{super::list_type_configuration::LIST_TYPE_CONFIGURATION, *};
    use crate::{
        hir_mir::{
            error_type_configuration::ERROR_TYPE_CONFIGURATION,
            string_type_configuration::STRING_TYPE_CONFIGURATION,
        },
        position::Position,
    };

    fn validate_module(module: &Module) -> Result<(), CompileError> {
        validate(
            module,
            &TypeContext::new(
                module,
                &LIST_TYPE_CONFIGURATION,
                &STRING_TYPE_CONFIGURATION,
                &ERROR_TYPE_CONFIGURATION,
            ),
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
                    .set_type_definitions(vec![TypeDefinition::without_source(
                        "error",
                        vec![],
                        false,
                        false,
                        false
                    )])
                    .set_definitions(vec![Definition::without_source(
                        "x",
                        Lambda::new(
                            vec![Argument::new(
                                "x",
                                types::Union::new(
                                    types::None::new(Position::dummy()),
                                    types::Reference::new(
                                        &ERROR_TYPE_CONFIGURATION.error_type_name,
                                        Position::dummy(),
                                    ),
                                    Position::dummy(),
                                ),
                            )],
                            types::None::new(Position::dummy()),
                            TryOperation::new(
                                None,
                                Variable::new("x", Position::dummy()),
                                Position::dummy(),
                            ),
                            Position::dummy(),
                        ),
                        false,
                    )])
            ),
            Err(CompileError::InvalidTryOperation(Position::dummy()))
        );
    }

    #[test]
    fn validate_thunk() {
        let error_type =
            types::Reference::new(&ERROR_TYPE_CONFIGURATION.error_type_name, Position::dummy());
        let union_type = types::Union::new(
            types::None::new(Position::dummy()),
            error_type.clone(),
            Position::dummy(),
        );

        assert_eq!(
            validate_module(
                &Module::empty()
                    .set_type_definitions(vec![TypeDefinition::without_source(
                        "error",
                        vec![],
                        false,
                        false,
                        false
                    )])
                    .set_definitions(vec![Definition::without_source(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", union_type.clone())],
                            types::Function::new(
                                vec![],
                                types::None::new(Position::dummy()),
                                Position::dummy()
                            ),
                            Thunk::new(
                                Some(union_type.into()),
                                TryOperation::new(
                                    None,
                                    Variable::new("x", Position::dummy()),
                                    Position::dummy(),
                                ),
                                Position::dummy()
                            ),
                            Position::dummy(),
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
            types::Reference::new(&ERROR_TYPE_CONFIGURATION.error_type_name, Position::dummy());

        assert_eq!(
            validate_module(
                &Module::empty()
                    .set_type_definitions(vec![TypeDefinition::without_source(
                        "error",
                        vec![],
                        false,
                        false,
                        false
                    )])
                    .set_definitions(vec![Definition::without_source(
                        "x",
                        Lambda::new(
                            vec![Argument::new(
                                "x",
                                types::Union::new(
                                    types::None::new(Position::dummy()),
                                    error_type.clone(),
                                    Position::dummy(),
                                ),
                            )],
                            types::Function::new(
                                vec![],
                                types::None::new(Position::dummy()),
                                Position::dummy()
                            ),
                            Thunk::new(
                                Some(types::None::new(Position::dummy()).into()),
                                TryOperation::new(
                                    None,
                                    Variable::new("x", Position::dummy()),
                                    Position::dummy(),
                                ),
                                Position::dummy()
                            ),
                            Position::dummy(),
                        ),
                        false,
                    )])
            ),
            Err(CompileError::InvalidTryOperation(Position::dummy()))
        );
    }
}
