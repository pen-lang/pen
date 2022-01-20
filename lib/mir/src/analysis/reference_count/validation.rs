use super::ReferenceCountError;
use crate::ir::{Definition, Expression, Module};
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub fn validate(module: &Module) -> Result<(), ReferenceCountError> {
    for definition in module.definitions() {
        validate_global_definition(definition)?;
    }

    Ok(())
}

fn validate_global_definition(definition: &Definition) -> Result<(), ReferenceCountError> {
    validate_definition_body(
        definition.body(),
        collect_definition_local_variables(definition)
            .into_iter()
            .map(|name| (name, 1))
            .collect(),
    )
}

fn validate_local_definition(definition: &Definition) -> Result<(), ReferenceCountError> {
    validate_definition_body(
        definition.body(),
        [definition.name().into()]
            .into_iter()
            .chain(collect_definition_local_variables(definition))
            .map(|name| (name, 1))
            .collect(),
    )
}

fn collect_definition_local_variables(definition: &Definition) -> BTreeSet<String> {
    definition
        .environment()
        .iter()
        .chain(definition.arguments())
        .map(|argument| argument.name().into())
        .collect()
}

fn validate_definition_body(
    body: &Expression,
    mut variables: HashMap<String, isize>,
) -> Result<(), ReferenceCountError> {
    move_expression(body, &mut variables);

    let invalid_variables = variables
        .into_iter()
        .filter(|(_, count)| count != &0)
        .collect::<BTreeMap<_, _>>();

    if !invalid_variables.is_empty() {
        return Err(ReferenceCountError::InvalidReferenceCount(
            invalid_variables,
        ));
    }

    Ok(())
}

fn move_expression(expression: &Expression, variables: &mut HashMap<String, isize>) {
    match expression {
        Expression::ArithmeticOperation(_) => todo!(),
        Expression::Boolean(_) => {}
        Expression::ByteString(_) => {}
        Expression::Call(_) => todo!(),
        Expression::Case(_) => todo!(),
        Expression::CloneVariables(_) => todo!(),
        Expression::ComparisonOperation(_) => todo!(),
        Expression::DropVariables(drop) => {
            for name in drop.variables().keys() {
                drop_variable(name, variables);
            }

            move_expression(drop.expression(), variables);
        }
        Expression::If(_) => todo!(),
        Expression::Let(_) => todo!(),
        Expression::LetRecursive(_) => todo!(),
        Expression::None => {}
        Expression::Number(_) => {}
        Expression::Record(_) => todo!(),
        Expression::RecordField(_) => todo!(),
        Expression::TryOperation(_) => todo!(),
        Expression::Variable(variable) => {
            drop_variable(variable.name(), variables);
        }
        Expression::Variant(_) => todo!(),
    }
}

fn drop_variable(name: impl AsRef<str>, variables: &mut HashMap<String, isize>) {
    variables.insert(name.as_ref().into(), variables[name.as_ref()] - 1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ir::{Argument, DropVariables, Variable},
        types::Type,
    };

    #[test]
    fn validate_empty_module() {
        validate(&Module::new(vec![], vec![], vec![], vec![], vec![])).unwrap();
    }

    #[test]
    fn validate_none() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new("f", vec![], Expression::None, Type::None)],
        ))
        .unwrap();
    }

    #[test]
    fn validate_variable_drop() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::None)],
                DropVariables::new(
                    [("x".into(), Type::None)].into_iter().collect(),
                    Expression::None,
                ),
                Type::None,
            )],
        ))
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn fail_to_validate_variable_drop() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::None)],
                Expression::None,
                Type::None,
            )],
        ))
        .unwrap();
    }

    #[test]
    fn validate_variable_move() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::None)],
                Variable::new("x"),
                Type::None,
            )],
        ))
        .unwrap();
    }
}
