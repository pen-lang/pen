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
    let validate_expression =
        |expression| validate_expression(expression, result_type, type_context);

    match expression {
        Expression::Call(call) => {
            validate_expression(call.function())?;

            for argument in call.arguments() {
                validate_expression(argument)?;
            }
        }
        Expression::If(if_) => {
            validate_expression(if_.condition())?;
            validate_expression(if_.then())?;
            validate_expression(if_.else_())?;
        }
        Expression::IfList(if_) => {
            validate_expression(if_.argument())?;
            validate_expression(if_.then())?;
            validate_expression(if_.else_())?;
        }
        Expression::IfType(if_) => {
            validate_expression(if_.argument())?;

            for branch in if_.branches() {
                validate_expression(branch.expression())?;
            }

            if let Some(branch) = if_.else_() {
                validate_expression(branch.expression())?;
            }
        }
        Expression::TypeCoercion(coercion) => {
            validate_expression(coercion.argument())?;
        }
        Expression::Lambda(lambda) => {
            validate_lambda(lambda, type_context)?;
        }
        Expression::Let(let_) => {
            validate_expression(let_.bound_expression())?;
            validate_expression(let_.expression())?;
        }
        Expression::List(list) => {
            for element in list.elements() {
                validate_expression(match element {
                    ListElement::Multiple(expression) => expression,
                    ListElement::Single(expression) => expression,
                })?;
            }
        }
        Expression::Operation(operation) => match operation {
            Operation::Arithmetic(operation) => {
                validate_expression(operation.lhs())?;
                validate_expression(operation.rhs())?;
            }
            Operation::Boolean(operation) => {
                validate_expression(operation.lhs())?;
                validate_expression(operation.rhs())?;
            }
            Operation::Equality(operation) => {
                validate_expression(operation.lhs())?;
                validate_expression(operation.rhs())?;
            }
            Operation::Not(operation) => {
                validate_expression(operation.expression())?;
            }
            Operation::Order(operation) => {
                validate_expression(operation.lhs())?;
                validate_expression(operation.rhs())?;
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

                validate_expression(operation.expression())?;
            }
        },
        Expression::RecordConstruction(construction) => {
            for element in construction.elements() {
                validate_expression(element.expression())?;
            }
        }
        Expression::RecordDeconstruction(deconstruction) => {
            validate_expression(deconstruction.record())?;
        }
        Expression::RecordUpdate(update) => {
            validate_expression(update.record())?;

            for element in update.elements() {
                validate_expression(element.expression())?;
            }
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
    use crate::hir_mir::{
        error_type_configuration::ERROR_TYPE_CONFIGURATION,
        string_type_configuration::STRING_TYPE_CONFIGURATION,
    };
    use crate::position::Position;

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
    fn fail_to_validate_try_operator() {
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
}
