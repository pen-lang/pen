use super::{
    environment, type_context::TypeContext, type_extraction, type_subsumption, CompileError,
};
use crate::{
    hir::*,
    types::{self, Type},
};
use std::collections::HashMap;

pub fn check_types(module: &Module, type_context: &TypeContext) -> Result<(), CompileError> {
    let variables = environment::create_from_module(module);

    for definition in module.definitions() {
        check_lambda(definition.lambda(), &variables, type_context)?;
    }

    Ok(())
}

fn check_lambda(
    lambda: &Lambda,
    variables: &HashMap<String, Type>,
    type_context: &TypeContext,
) -> Result<types::Function, CompileError> {
    check_subsumption(
        &check_block(
            lambda.body(),
            &variables
                .clone()
                .into_iter()
                .chain(
                    lambda
                        .arguments()
                        .iter()
                        .map(|argument| (argument.name().into(), argument.type_().clone())),
                )
                .collect(),
            type_context,
        )?,
        &lambda.result_type(),
        type_context.types(),
    )?;

    Ok(type_extraction::extract_from_lambda(lambda))
}

fn check_expression(
    expression: &Expression,
    variables: &HashMap<String, Type>,
    type_context: &TypeContext,
) -> Result<Type, CompileError> {
    Ok(match expression {
        Expression::Boolean(boolean) => types::Boolean::new(boolean.position().clone()).into(),
        Expression::Lambda(lambda) => check_lambda(lambda, variables, type_context)?.into(),
        Expression::None(none) => types::None::new(none.position().clone()).into(),
        Expression::Number(number) => types::Number::new(number.position().clone()).into(),
        Expression::String(string) => types::ByteString::new(string.position().clone()).into(),
        Expression::Variable(variable) => variables
            .get(variable.name())
            .ok_or_else(|| CompileError::VariableNotFound(variable.clone()))?
            .clone(),
        _ => todo!(),
    })
}

fn check_block(
    block: &Block,
    variables: &HashMap<String, Type>,
    type_context: &TypeContext,
) -> Result<Type, CompileError> {
    let mut variables = variables.clone();

    for assignment in block.assignments() {
        check_subsumption(
            &check_expression(assignment.expression(), &variables, type_context)?,
            assignment
                .type_()
                .ok_or_else(|| CompileError::TypeNotInferred(assignment.position().clone()))?,
            type_context.types(),
        )?;

        variables.insert(
            assignment.name().into(),
            assignment
                .type_()
                .cloned()
                .ok_or_else(|| CompileError::TypeNotInferred(assignment.position().clone()))?,
        );
    }

    check_expression(block.expression(), &variables, type_context)
}

fn check_subsumption(
    lower: &Type,
    upper: &Type,
    types: &HashMap<String, Type>,
) -> Result<(), CompileError> {
    if type_subsumption::check_subsumption(lower, upper, types)? {
        Ok(())
    } else {
        Err(CompileError::TypesNotMatched(
            lower.position().clone(),
            upper.position().clone(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{compile::list_type_configuration::LIST_TYPE_CONFIGURATION, position::Position};

    fn check_module(module: &Module) -> Result<(), CompileError> {
        check_types(module, &TypeContext::new(module, &LIST_TYPE_CONFIGURATION))
    }

    #[test]
    fn check_empty_module() -> Result<(), CompileError> {
        check_module(&Module::new(vec![], vec![], vec![], vec![]))
    }

    #[test]
    fn check_definition() -> Result<(), CompileError> {
        check_module(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::dummy()),
                    Block::new(vec![], None::new(Position::dummy())),
                    Position::dummy(),
                ),
                false,
                Position::dummy(),
            )],
        ))
    }

    #[test]
    fn check_assignment() -> Result<(), CompileError> {
        check_module(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::dummy()),
                    Block::new(
                        vec![Assignment::new(
                            "y",
                            None::new(Position::dummy()),
                            Some(types::None::new(Position::dummy()).into()),
                            Position::dummy(),
                        )],
                        Variable::new("y", Position::dummy()),
                    ),
                    Position::dummy(),
                ),
                false,
                Position::dummy(),
            )],
        ))
    }

    #[test]
    fn check_subsumption_in_lambda() -> Result<(), CompileError> {
        check_module(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "x",
                Lambda::new(
                    vec![],
                    types::Union::new(
                        types::Number::new(Position::dummy()),
                        types::None::new(Position::dummy()),
                        Position::dummy(),
                    ),
                    Block::new(vec![], None::new(Position::dummy())),
                    Position::dummy(),
                ),
                false,
                Position::dummy(),
            )],
        ))
    }
}
