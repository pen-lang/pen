use super::ReferenceCountError;
use crate::ir::{Definition, Expression, Module};
use std::collections::{BTreeSet, HashMap};

pub fn validate(module: &Module) -> Result<(), ReferenceCountError> {
    for definition in module.definitions() {
        validate_global_definition(definition)?;
    }

    Ok(())
}

fn validate_global_definition(definition: &Definition) -> Result<(), ReferenceCountError> {
    validate_definition_body_variables(&move_expression(
        definition.body(),
        &collect_definition_local_variables(definition)
            .into_iter()
            .map(|name| (name, 1))
            .collect(),
    ))
}

fn validate_local_definition(definition: &Definition) -> Result<(), ReferenceCountError> {
    validate_definition_body_variables(&move_expression(
        definition.body(),
        &[definition.name().into()]
            .into_iter()
            .chain(collect_definition_local_variables(definition))
            .map(|name| (name, 1))
            .collect(),
    ))
}

fn collect_definition_local_variables(definition: &Definition) -> BTreeSet<String> {
    definition
        .environment()
        .iter()
        .chain(definition.arguments())
        .map(|argument| argument.name().into())
        .collect()
}

fn validate_definition_body_variables(
    variables: &HashMap<String, isize>,
) -> Result<(), ReferenceCountError> {
    let invalid_variables = variables
        .iter()
        .filter_map(
            |(name, &count)| {
                if count != 0 {
                    Some(name.clone())
                } else {
                    None
                }
            },
        )
        .collect::<BTreeSet<_>>();

    if !invalid_variables.is_empty() {
        return Err(ReferenceCountError::InvalidReferenceCount(
            invalid_variables,
        ));
    }

    Ok(())
}

fn move_expression(
    expression: &Expression,
    variables: &HashMap<String, isize>,
) -> HashMap<String, isize> {
    match expression {
        Expression::ArithmeticOperation(_) => todo!(),
        Expression::Boolean(_) => todo!(),
        Expression::ByteString(_) => variables.clone(),
        Expression::Call(_) => todo!(),
        Expression::Case(_) => todo!(),
        Expression::CloneVariables(_) => todo!(),
        Expression::ComparisonOperation(_) => todo!(),
        Expression::DropVariables(_) => todo!(),
        Expression::If(_) => todo!(),
        Expression::Let(_) => todo!(),
        Expression::LetRecursive(_) => todo!(),
        Expression::None => variables.clone(),
        Expression::Number(_) => variables.clone(),
        Expression::Record(_) => todo!(),
        Expression::RecordField(_) => todo!(),
        Expression::TryOperation(_) => todo!(),
        Expression::Variable(_) => todo!(),
        Expression::Variant(_) => todo!(),
    }
}
