use super::{environment_creator, type_context::TypeContext, type_extractor, CompileError};
use crate::{
    hir::*,
    types::{
        self,
        analysis::{
            type_canonicalizer, type_equality_checker, type_resolver, type_subsumption_checker,
            union_type_creator,
        },
        Type,
    },
};
use std::collections::{HashMap, HashSet};

pub fn check_types(module: &Module, type_context: &TypeContext) -> Result<(), CompileError> {
    let variables = environment_creator::create_from_module(module);

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
        &check_expression(
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
        lambda.result_type(),
        type_context.types(),
    )?;

    Ok(type_extractor::extract_from_lambda(lambda))
}

fn check_expression(
    expression: &Expression,
    variables: &HashMap<String, Type>,
    type_context: &TypeContext,
) -> Result<Type, CompileError> {
    let check_expression =
        |expression, variables: &_| check_expression(expression, variables, type_context);
    let check_subsumption =
        |lower: &_, upper| check_subsumption(lower, upper, type_context.types());

    Ok(match expression {
        Expression::Boolean(boolean) => types::Boolean::new(boolean.position().clone()).into(),
        Expression::Call(call) => {
            let type_ = call
                .function_type()
                .ok_or_else(|| CompileError::TypeNotInferred(call.position().clone()))?;
            let function_type = type_resolver::resolve_to_function(type_, type_context.types())?
                .ok_or_else(|| {
                    CompileError::FunctionExpected(call.function().position().clone())
                })?;

            check_subsumption(&check_expression(call.function(), variables)?, type_)?;

            if call.arguments().len() != function_type.arguments().len() {
                return Err(CompileError::WrongArgumentCount(call.position().clone()));
            }

            for (argument, type_) in call.arguments().iter().zip(function_type.arguments()) {
                check_subsumption(&check_expression(argument, variables)?, type_)?;
            }

            function_type.result().clone()
        }
        Expression::If(if_) => {
            check_subsumption(
                &check_expression(if_.condition(), variables)?,
                &types::Boolean::new(if_.position().clone()).into(),
            )?;

            type_extractor::extract_from_expression(expression, variables, type_context)?
        }
        Expression::IfType(if_) => {
            let argument_type = type_canonicalizer::canonicalize(
                &check_expression(if_.argument(), variables)?,
                type_context.types(),
            )?;

            if !matches!(argument_type, Type::Union(_) | Type::Any(_)) {
                return Err(CompileError::UnionOrAnyTypeExpected(
                    if_.argument().position().clone(),
                ));
            }

            for branch in if_.branches() {
                check_expression(
                    branch.expression(),
                    &variables
                        .clone()
                        .into_iter()
                        .chain(vec![(if_.name().into(), branch.type_().clone())])
                        .collect(),
                )?;
            }

            if let Some(expression) = if_.else_() {
                check_expression(
                    expression,
                    &variables
                        .clone()
                        .into_iter()
                        .chain(vec![(if_.name().into(), argument_type.clone())])
                        .collect(),
                )?;
            } else if !type_equality_checker::check_equality(
                &argument_type,
                &union_type_creator::create_union_type(
                    &if_.branches()
                        .iter()
                        .map(|branch| branch.type_().clone())
                        .collect::<Vec<_>>(),
                    if_.position(),
                )
                .unwrap(),
                type_context.types(),
            )? {
                return Err(CompileError::MissingElseBlock(if_.position().clone()));
            }

            type_extractor::extract_from_expression(expression, variables, type_context)?
        }
        Expression::Lambda(lambda) => check_lambda(lambda, variables, type_context)?.into(),
        Expression::None(none) => types::None::new(none.position().clone()).into(),
        Expression::Number(number) => types::Number::new(number.position().clone()).into(),
        Expression::Operation(operation) => check_operation(operation, variables, type_context)?,
        Expression::RecordConstruction(construction) => {
            let element_types = type_resolver::resolve_record_elements(
                construction.type_(),
                construction.position(),
                type_context.types(),
                type_context.records(),
            )?;

            for element in construction.elements() {
                check_subsumption(
                    &check_expression(element.expression(), variables)?,
                    element_types
                        .iter()
                        .find(|element_type| element_type.name() == element.name())
                        .ok_or_else(|| {
                            CompileError::RecordElementUnknown(expression.position().clone())
                        })?
                        .type_(),
                )?;
            }

            let element_names = construction
                .elements()
                .iter()
                .map(|element| element.name())
                .collect::<HashSet<_>>();

            for element_type in element_types {
                if !element_names.contains(element_type.name()) {
                    return Err(CompileError::RecordElementMissing(
                        construction.position().clone(),
                    ));
                }
            }

            construction.type_().clone()
        }
        Expression::RecordDeconstruction(deconstruction) => {
            let type_ = deconstruction
                .type_()
                .ok_or_else(|| CompileError::TypeNotInferred(deconstruction.position().clone()))?;

            check_subsumption(
                &check_expression(deconstruction.record(), variables)?,
                type_,
            )?;

            let element_types = type_resolver::resolve_record_elements(
                type_,
                deconstruction.position(),
                type_context.types(),
                type_context.records(),
            )?;

            element_types
                .iter()
                .find(|element_type| element_type.name() == deconstruction.element_name())
                .ok_or_else(|| {
                    CompileError::RecordElementUnknown(deconstruction.position().clone())
                })?
                .type_()
                .clone()
        }
        Expression::RecordUpdate(update) => {
            check_subsumption(
                &check_expression(update.record(), variables)?,
                update.type_(),
            )?;

            let element_types = type_resolver::resolve_record_elements(
                update.type_(),
                update.position(),
                type_context.types(),
                type_context.records(),
            )?;

            for element in update.elements() {
                check_subsumption(
                    &check_expression(element.expression(), variables)?,
                    element_types
                        .iter()
                        .find(|element_type| element_type.name() == element.name())
                        .ok_or_else(|| {
                            CompileError::RecordElementUnknown(expression.position().clone())
                        })?
                        .type_(),
                )?;
            }

            update.type_().clone()
        }
        Expression::String(string) => types::ByteString::new(string.position().clone()).into(),
        Expression::Variable(variable) => variables
            .get(variable.name())
            .ok_or_else(|| CompileError::VariableNotFound(variable.clone()))?
            .clone(),
        _ => todo!(),
    })
}

fn check_operation(
    operation: &Operation,
    variables: &HashMap<String, Type>,
    type_context: &TypeContext,
) -> Result<Type, CompileError> {
    let check_expression = |expression| check_expression(expression, variables, type_context);
    let check_subsumption =
        |lower: &_, upper| check_subsumption(lower, upper, type_context.types());

    Ok(match operation {
        Operation::Arithmetic(operation) => {
            let number_type = types::Number::new(operation.position().clone()).into();

            check_subsumption(&check_expression(operation.lhs())?, &number_type)?;
            check_subsumption(&check_expression(operation.rhs())?, &number_type)?;

            number_type
        }
        Operation::Boolean(operation) => {
            let boolean_type = types::Boolean::new(operation.position().clone()).into();

            check_subsumption(&check_expression(operation.lhs())?, &boolean_type)?;
            check_subsumption(&check_expression(operation.rhs())?, &boolean_type)?;

            boolean_type
        }
        Operation::Not(operation) => {
            let boolean_type = types::Boolean::new(operation.position().clone()).into();

            check_subsumption(&check_expression(operation.expression())?, &boolean_type)?;

            boolean_type
        }
        Operation::Order(operation) => {
            let number_type = types::Number::new(operation.position().clone()).into();

            check_subsumption(&check_expression(operation.lhs())?, &number_type)?;
            check_subsumption(&check_expression(operation.rhs())?, &number_type)?;

            types::Boolean::new(operation.position().clone()).into()
        }
        _ => todo!(),
    })
}

fn check_subsumption(
    lower: &Type,
    upper: &Type,
    types: &HashMap<String, Type>,
) -> Result<(), CompileError> {
    if type_subsumption_checker::check_subsumption(lower, upper, types)? {
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
    use super::{super::list_type_configuration::LIST_TYPE_CONFIGURATION, *};
    use crate::{
        hir_mir::string_type_configuration::STRING_TYPE_CONFIGURATION, position::Position,
    };

    fn check_module(module: &Module) -> Result<(), CompileError> {
        check_types(
            module,
            &TypeContext::new(module, &LIST_TYPE_CONFIGURATION, &STRING_TYPE_CONFIGURATION),
        )
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
            vec![Definition::without_source(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::dummy()),
                    None::new(Position::dummy()),
                    Position::dummy(),
                ),
                false,
            )],
        ))
    }

    #[test]
    fn check_subsumption_of_function_result_type() -> Result<(), CompileError> {
        check_module(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![Definition::without_source(
                "x",
                Lambda::new(
                    vec![],
                    types::Union::new(
                        types::Number::new(Position::dummy()),
                        types::None::new(Position::dummy()),
                        Position::dummy(),
                    ),
                    None::new(Position::dummy()),
                    Position::dummy(),
                ),
                false,
            )],
        ))
    }

    mod if_ {
        use super::*;

        #[test]
        fn check_if() {
            check_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "f",
                    Lambda::new(
                        vec![],
                        types::Number::new(Position::dummy()),
                        If::new(
                            Boolean::new(true, Position::dummy()),
                            Number::new(0.0, Position::dummy()),
                            Number::new(0.0, Position::dummy()),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
            .unwrap()
        }

        #[test]
        fn check_if_of_union_type() {
            check_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "f",
                    Lambda::new(
                        vec![],
                        types::Union::new(
                            types::Number::new(Position::dummy()),
                            types::None::new(Position::dummy()),
                            Position::dummy(),
                        ),
                        If::new(
                            Boolean::new(true, Position::dummy()),
                            Number::new(0.0, Position::dummy()),
                            None::new(Position::dummy()),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
            .unwrap()
        }
    }

    mod if_type {
        use super::*;

        #[test]
        fn check_with_union() {
            let union_type = types::Union::new(
                types::Number::new(Position::dummy()),
                types::None::new(Position::dummy()),
                Position::dummy(),
            );

            check_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", union_type.clone())],
                        types::None::new(Position::dummy()),
                        IfType::new(
                            "y",
                            Variable::new("x", Position::dummy()),
                            vec![
                                IfTypeBranch::new(
                                    types::Number::new(Position::dummy()),
                                    None::new(Position::dummy()),
                                ),
                                IfTypeBranch::new(
                                    types::None::new(Position::dummy()),
                                    None::new(Position::dummy()),
                                ),
                            ],
                            None,
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
            .unwrap()
        }

        #[test]
        fn check_with_any() {
            check_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::Any::new(Position::dummy()))],
                        types::None::new(Position::dummy()),
                        IfType::new(
                            "y",
                            Variable::new("x", Position::dummy()),
                            vec![IfTypeBranch::new(
                                types::None::new(Position::dummy()),
                                None::new(Position::dummy()),
                            )],
                            Some(None::new(Position::dummy()).into()),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
            .unwrap()
        }

        #[test]
        fn check_result_of_union() {
            check_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::Any::new(Position::dummy()))],
                        types::Union::new(
                            types::Number::new(Position::dummy()),
                            types::None::new(Position::dummy()),
                            Position::dummy(),
                        ),
                        IfType::new(
                            "y",
                            Variable::new("x", Position::dummy()),
                            vec![IfTypeBranch::new(
                                types::None::new(Position::dummy()),
                                Number::new(42.0, Position::dummy()),
                            )],
                            Some(None::new(Position::dummy()).into()),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
            .unwrap()
        }

        #[test]
        fn check_result_of_any() {
            check_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::Any::new(Position::dummy()))],
                        types::Any::new(Position::dummy()),
                        IfType::new(
                            "y",
                            Variable::new("x", Position::dummy()),
                            vec![IfTypeBranch::new(
                                types::None::new(Position::dummy()),
                                None::new(Position::dummy()),
                            )],
                            Some(Variable::new("y", Position::dummy()).into()),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
            .unwrap()
        }

        #[test]
        #[should_panic]
        fn fail_to_check_due_to_wrong_argument_type() {
            check_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "f",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::dummy()),
                        IfType::new(
                            "y",
                            None::new(Position::dummy()),
                            vec![IfTypeBranch::new(
                                types::None::new(Position::dummy()),
                                None::new(Position::dummy()),
                            )],
                            Some(None::new(Position::dummy()).into()),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
            .unwrap()
        }

        #[test]
        #[should_panic]
        fn fail_to_check_union_due_to_missing_else() {
            check_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "f",
                    Lambda::new(
                        vec![Argument::new(
                            "x",
                            types::Union::new(
                                types::Number::new(Position::dummy()),
                                types::None::new(Position::dummy()),
                                Position::dummy(),
                            ),
                        )],
                        types::None::new(Position::dummy()),
                        IfType::new(
                            "y",
                            Variable::new("x", Position::dummy()),
                            vec![IfTypeBranch::new(
                                types::Number::new(Position::dummy()),
                                None::new(Position::dummy()),
                            )],
                            None,
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
            .unwrap();
        }

        #[test]
        #[should_panic]
        fn fail_to_check_any_due_to_missing_else() {
            check_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "f",
                    Lambda::new(
                        vec![Argument::new(
                            "x",
                            types::Union::new(
                                types::Number::new(Position::dummy()),
                                types::None::new(Position::dummy()),
                                Position::dummy(),
                            ),
                        )],
                        types::None::new(Position::dummy()),
                        IfType::new(
                            "y",
                            Variable::new("x", Position::dummy()),
                            vec![IfTypeBranch::new(
                                types::Number::new(Position::dummy()),
                                None::new(Position::dummy()),
                            )],
                            None,
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
            .unwrap();
        }
    }

    mod calls {
        use super::*;

        #[test]
        fn check_call() {
            check_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "f",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::dummy()),
                        Call::new(
                            Variable::new("f", Position::dummy()),
                            vec![],
                            Some(
                                types::Function::new(
                                    vec![],
                                    types::None::new(Position::dummy()),
                                    Position::dummy(),
                                )
                                .into(),
                            ),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
            .unwrap()
        }

        #[test]
        fn check_call_with_arguments() {
            check_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::None::new(Position::dummy()))],
                        types::None::new(Position::dummy()),
                        Call::new(
                            Variable::new("f", Position::dummy()),
                            vec![None::new(Position::dummy()).into()],
                            Some(
                                types::Function::new(
                                    vec![types::None::new(Position::dummy()).into()],
                                    types::None::new(Position::dummy()),
                                    Position::dummy(),
                                )
                                .into(),
                            ),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
            .unwrap()
        }

        #[test]
        #[should_panic]
        fn fail_to_check_call_with_wrong_argument_type() {
            check_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::None::new(Position::dummy()))],
                        types::None::new(Position::dummy()),
                        Call::new(
                            Variable::new("f", Position::dummy()),
                            vec![Number::new(42.0, Position::dummy()).into()],
                            Some(
                                types::Function::new(
                                    vec![types::None::new(Position::dummy()).into()],
                                    types::None::new(Position::dummy()),
                                    Position::dummy(),
                                )
                                .into(),
                            ),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
            .unwrap()
        }

        #[test]
        #[should_panic]
        fn fail_to_check_call_with_wrong_argument_count() {
            check_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::None::new(Position::dummy()))],
                        types::None::new(Position::dummy()),
                        Call::new(
                            Variable::new("f", Position::dummy()),
                            vec![],
                            Some(
                                types::Function::new(
                                    vec![types::None::new(Position::dummy()).into()],
                                    types::None::new(Position::dummy()),
                                    Position::dummy(),
                                )
                                .into(),
                            ),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
            .unwrap()
        }
    }

    mod operations {
        use super::*;

        #[test]
        fn check_arithmetic_operation() {
            check_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![],
                        types::Number::new(Position::dummy()),
                        ArithmeticOperation::new(
                            ArithmeticOperator::Add,
                            Number::new(0.0, Position::dummy()),
                            Number::new(0.0, Position::dummy()),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
            .unwrap();
        }

        #[test]
        fn check_boolean_operation() {
            check_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![],
                        types::Boolean::new(Position::dummy()),
                        BooleanOperation::new(
                            BooleanOperator::And,
                            Boolean::new(true, Position::dummy()),
                            Boolean::new(true, Position::dummy()),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
            .unwrap();
        }

        #[test]
        fn fail_to_check_boolean_operation() {
            assert_eq!(
                check_module(&Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![Definition::without_source(
                        "x",
                        Lambda::new(
                            vec![],
                            types::Boolean::new(Position::dummy()),
                            BooleanOperation::new(
                                BooleanOperator::And,
                                Number::new(42.0, Position::dummy()),
                                Boolean::new(true, Position::dummy()),
                                Position::dummy(),
                            ),
                            Position::dummy(),
                        ),
                        false,
                    )],
                )),
                Err(CompileError::TypesNotMatched(
                    Position::dummy(),
                    Position::dummy()
                ))
            );
        }

        #[test]
        fn check_not_operation() {
            check_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![],
                        types::Boolean::new(Position::dummy()),
                        NotOperation::new(Boolean::new(true, Position::dummy()), Position::dummy()),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
            .unwrap();
        }

        #[test]
        fn check_order_operation() {
            check_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![],
                        types::Boolean::new(Position::dummy()),
                        OrderOperation::new(
                            OrderOperator::LessThan,
                            Number::new(0.0, Position::dummy()),
                            Number::new(0.0, Position::dummy()),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
            .unwrap();
        }
    }

    mod records {
        use super::*;

        #[test]
        fn check_record() -> Result<(), CompileError> {
            let reference_type = types::Reference::new("r", Position::dummy());

            check_module(&Module::new(
                vec![TypeDefinition::without_source(
                    "r",
                    vec![types::RecordElement::new(
                        "x",
                        types::None::new(Position::dummy()),
                    )],
                    false,
                    false,
                    false,
                )],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![],
                        reference_type.clone(),
                        RecordConstruction::new(
                            reference_type,
                            vec![RecordElement::new(
                                "x",
                                None::new(Position::dummy()),
                                Position::dummy(),
                            )],
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
        }

        #[test]
        fn fail_to_check_record_with_missing_element() {
            let reference_type = types::Reference::new("r", Position::dummy());

            assert!(matches!(
                check_module(&Module::new(
                    vec![TypeDefinition::without_source(
                        "r",
                        vec![types::RecordElement::new(
                            "x",
                            types::None::new(Position::dummy()),
                        )],
                        false,
                        false,
                        false
                    )],
                    vec![],
                    vec![],
                    vec![Definition::without_source(
                        "x",
                        Lambda::new(
                            vec![],
                            reference_type.clone(),
                            RecordConstruction::new(
                                reference_type,
                                Default::default(),
                                Position::dummy(),
                            ),
                            Position::dummy(),
                        ),
                        false
                    )],
                )),
                Err(CompileError::RecordElementMissing(_))
            ));
        }

        #[test]
        fn fail_to_check_record_with_unknown_element() {
            let reference_type = types::Reference::new("r", Position::dummy());

            assert!(matches!(
                check_module(&Module::new(
                    vec![TypeDefinition::without_source(
                        "r",
                        vec![],
                        false,
                        false,
                        false
                    )],
                    vec![],
                    vec![],
                    vec![Definition::without_source(
                        "x",
                        Lambda::new(
                            vec![],
                            reference_type.clone(),
                            RecordConstruction::new(
                                reference_type,
                                vec![RecordElement::new(
                                    "x",
                                    None::new(Position::dummy()),
                                    Position::dummy()
                                )],
                                Position::dummy(),
                            ),
                            Position::dummy(),
                        ),
                        false
                    )],
                )),
                Err(CompileError::RecordElementUnknown(_))
            ));
        }

        #[test]
        fn check_record_update() -> Result<(), CompileError> {
            let reference_type = types::Reference::new("r", Position::dummy());

            check_module(&Module::new(
                vec![TypeDefinition::without_source(
                    "r",
                    vec![types::RecordElement::new(
                        "x",
                        types::None::new(Position::dummy()),
                    )],
                    false,
                    false,
                    false,
                )],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", reference_type.clone())],
                        reference_type.clone(),
                        RecordUpdate::new(
                            reference_type,
                            Variable::new("x", Position::dummy()),
                            vec![RecordElement::new(
                                "x",
                                None::new(Position::dummy()),
                                Position::dummy(),
                            )],
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
        }

        #[test]
        fn check_record_deconstruction() -> Result<(), CompileError> {
            let reference_type = types::Reference::new("r", Position::dummy());

            check_module(&Module::new(
                vec![TypeDefinition::without_source(
                    "r",
                    vec![types::RecordElement::new(
                        "x",
                        types::None::new(Position::dummy()),
                    )],
                    false,
                    false,
                    false,
                )],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", reference_type.clone())],
                        types::None::new(Position::dummy()),
                        RecordDeconstruction::new(
                            Some(reference_type.into()),
                            Variable::new("x", Position::dummy()),
                            "x",
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
        }

        #[test]
        fn fail_to_check_record_deconstruction_due_to_unknown_element() {
            let reference_type = types::Reference::new("r", Position::dummy());

            assert_eq!(
                check_module(&Module::new(
                    vec![TypeDefinition::without_source(
                        "r",
                        vec![types::RecordElement::new(
                            "x",
                            types::None::new(Position::dummy()),
                        )],
                        false,
                        false,
                        false,
                    )],
                    vec![],
                    vec![],
                    vec![Definition::without_source(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", reference_type.clone())],
                            types::None::new(Position::dummy()),
                            RecordDeconstruction::new(
                                Some(reference_type.into()),
                                Variable::new("x", Position::dummy()),
                                "y",
                                Position::dummy(),
                            ),
                            Position::dummy(),
                        ),
                        false,
                    )],
                )),
                Err(CompileError::RecordElementUnknown(Position::dummy()))
            );
        }
    }
}
