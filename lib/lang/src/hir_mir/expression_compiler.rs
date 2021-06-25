use super::{
    transformation::{
        boolean_operation_transformer, equal_operation_transformer, not_equal_operation_transformer,
    },
    type_compiler,
    type_context::TypeContext,
    CompileError,
};
use crate::{
    hir::*,
    types::{
        self,
        analysis::{type_canonicalizer, type_resolver},
        Type,
    },
};
use std::collections::HashMap;

const CLOSURE_NAME: &str = "$closure";

pub fn compile(
    expression: &Expression,
    type_context: &TypeContext,
) -> Result<mir::ir::Expression, CompileError> {
    let compile = |expression| compile(expression, type_context);

    Ok(match expression {
        Expression::Boolean(boolean) => mir::ir::Expression::Boolean(boolean.value()),
        Expression::Call(call) => mir::ir::Call::new(
            type_compiler::compile(
                call.function_type()
                    .ok_or_else(|| CompileError::TypeNotInferred(call.position().clone()))?,
                type_context,
            )?
            .into_function()
            .ok_or_else(|| CompileError::FunctionExpected(call.position().clone()))?,
            compile(call.function())?,
            call.arguments()
                .iter()
                .map(compile)
                .collect::<Result<_, _>>()?,
        )
        .into(),
        Expression::If(if_) => mir::ir::If::new(
            compile(if_.condition())?,
            compile(if_.then())?,
            compile(if_.else_())?,
        )
        .into(),
        Expression::Lambda(lambda) => mir::ir::LetRecursive::new(
            mir::ir::Definition::new(
                CLOSURE_NAME,
                lambda
                    .arguments()
                    .iter()
                    .map(|argument| -> Result<_, CompileError> {
                        Ok(mir::ir::Argument::new(
                            argument.name(),
                            type_compiler::compile(argument.type_(), type_context)?,
                        ))
                    })
                    .collect::<Result<_, _>>()?,
                compile(lambda.body())?,
                type_compiler::compile(lambda.result_type(), type_context)?,
            ),
            mir::ir::Variable::new(CLOSURE_NAME),
        )
        .into(),
        Expression::None(_) => mir::ir::Expression::None,
        Expression::Number(number) => mir::ir::Expression::Number(number.value()),
        Expression::Operation(operation) => compile_operation(operation, type_context)?,
        Expression::RecordConstruction(construction) => {
            let element_types = type_resolver::resolve_record_elements(
                construction.type_(),
                construction.position(),
                type_context.types(),
                type_context.records(),
            )?;
            let record_type = type_compiler::compile(construction.type_(), type_context)?
                .into_record()
                .unwrap();

            compile_record_elements(
                construction.elements(),
                element_types,
                &|elements| {
                    mir::ir::Record::new(
                        record_type.clone(),
                        element_types
                            .iter()
                            .map(|element_type| elements[element_type.name()].clone())
                            .collect(),
                    )
                    .into()
                },
                type_context,
            )?
        }
        Expression::String(string) => mir::ir::ByteString::new(string.value()).into(),
        Expression::TypeCoercion(coercion) => {
            let from = type_canonicalizer::canonicalize(coercion.from(), type_context.types())?;
            let to = type_canonicalizer::canonicalize(coercion.to(), type_context.types())?;
            let argument = compile(coercion.argument())?;

            if from.is_list() && to.is_list() {
                argument
            } else {
                match &from {
                    Type::Boolean(_)
                    | Type::None(_)
                    | Type::Number(_)
                    | Type::Record(_)
                    | Type::String(_) => mir::ir::Variant::new(
                        type_compiler::compile(coercion.from(), type_context)?,
                        argument,
                    )
                    .into(),
                    Type::Function(_) => todo!(),
                    Type::List(list_type) => {
                        let concrete_list_type =
                            type_compiler::compile_concrete_list(list_type, type_context.types())?;

                        mir::ir::Variant::new(
                            concrete_list_type.clone(),
                            mir::ir::Record::new(concrete_list_type, vec![argument]),
                        )
                        .into()
                    }
                    Type::Any(_) | Type::Union(_) => argument,
                    Type::Reference(_) => unreachable!(),
                }
            }
        }
        Expression::Variable(variable) => mir::ir::Variable::new(variable.name()).into(),
        _ => todo!(),
    })
}

fn compile_operation(
    operation: &Operation,
    type_context: &TypeContext,
) -> Result<mir::ir::Expression, CompileError> {
    let compile = |expression| compile(expression, type_context);

    Ok(match operation {
        Operation::Arithmetic(operation) => mir::ir::ArithmeticOperation::new(
            match operation.operator() {
                ArithmeticOperator::Add => mir::ir::ArithmeticOperator::Add,
                ArithmeticOperator::Subtract => mir::ir::ArithmeticOperator::Subtract,
                ArithmeticOperator::Multiply => mir::ir::ArithmeticOperator::Multiply,
                ArithmeticOperator::Divide => mir::ir::ArithmeticOperator::Divide,
            },
            compile(operation.lhs())?,
            compile(operation.rhs())?,
        )
        .into(),
        Operation::Boolean(operation) => {
            compile(&boolean_operation_transformer::transform(operation))?
        }
        Operation::Equality(operation) => match operation.operator() {
            EqualityOperator::Equal => {
                match type_resolver::resolve_type(
                    operation.type_().ok_or_else(|| {
                        CompileError::TypeNotInferred(operation.position().clone())
                    })?,
                    type_context.types(),
                )? {
                    Type::Number(_) => mir::ir::ComparisonOperation::new(
                        mir::ir::ComparisonOperator::Equal,
                        compile(operation.lhs())?,
                        compile(operation.rhs())?,
                    )
                    .into(),
                    Type::String(_) => mir::ir::Call::new(
                        mir::types::Function::new(
                            vec![mir::types::Type::ByteString, mir::types::Type::ByteString],
                            mir::types::Type::Boolean,
                        ),
                        mir::ir::Variable::new(
                            &type_context.string_type_configuration().equal_function_name,
                        ),
                        vec![compile(operation.lhs())?, compile(operation.rhs())?],
                    )
                    .into(),
                    _ => compile(&equal_operation_transformer::transform(operation)?)?,
                }
            }
            EqualityOperator::NotEqual => {
                compile(&not_equal_operation_transformer::transform(operation))?
            }
        },
        Operation::Not(operation) => {
            mir::ir::If::new(compile(operation.expression())?, false, true).into()
        }
        Operation::Order(operation) => mir::ir::ComparisonOperation::new(
            match operation.operator() {
                OrderOperator::LessThan => mir::ir::ComparisonOperator::LessThan,
                OrderOperator::LessThanOrEqual => mir::ir::ComparisonOperator::LessThanOrEqual,
                OrderOperator::GreaterThan => mir::ir::ComparisonOperator::GreaterThan,
                OrderOperator::GreaterThanOrEqual => {
                    mir::ir::ComparisonOperator::GreaterThanOrEqual
                }
            },
            compile(operation.lhs())?,
            compile(operation.rhs())?,
        )
        .into(),
        Operation::Try(_) => todo!(),
    })
}

fn compile_record_elements(
    elements: &[RecordElement],
    element_types: &[types::RecordElement],
    convert_elements_to_expression: &dyn Fn(
        &HashMap<String, mir::ir::Expression>,
    ) -> mir::ir::Expression,
    type_context: &TypeContext,
) -> Result<mir::ir::Expression, CompileError> {
    Ok(match elements {
        [] => convert_elements_to_expression(&Default::default()),
        [element, ..] => {
            let element_name = format!("${}", element.name());

            mir::ir::Let::new(
                element_name.clone(),
                type_compiler::compile(
                    element_types
                        .iter()
                        .find(|element_type| element_type.name() == element.name())
                        .ok_or_else(|| {
                            CompileError::RecordElementUnknown(element.position().clone())
                        })?
                        .type_(),
                    type_context,
                )?,
                compile(element.expression(), type_context)?,
                compile_record_elements(
                    &elements[1..],
                    element_types,
                    &|elements| {
                        convert_elements_to_expression(
                            &elements
                                .clone()
                                .into_iter()
                                .chain(vec![(
                                    element.name().into(),
                                    mir::ir::Variable::new(element_name.clone()).into(),
                                )])
                                .collect(),
                        )
                    },
                    type_context,
                )?,
            )
            .into()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::Position;
    use pretty_assertions::assert_eq;

    #[test]
    fn compile_record_construction() {
        assert_eq!(
            compile(
                &RecordConstruction::new(
                    types::Record::new("r", Position::dummy()),
                    vec![RecordElement::new(
                        "x",
                        None::new(Position::dummy()),
                        Position::dummy()
                    )],
                    Position::dummy()
                )
                .into(),
                &TypeContext::dummy(
                    vec![(
                        "r".into(),
                        vec![types::RecordElement::new(
                            "x",
                            types::None::new(Position::dummy())
                        )]
                    )]
                    .into_iter()
                    .collect(),
                    Default::default()
                ),
            ),
            Ok(mir::ir::Let::new(
                "$x",
                mir::types::Type::None,
                mir::ir::Expression::None,
                mir::ir::Record::new(
                    mir::types::Record::new("r"),
                    vec![mir::ir::Variable::new("$x").into()]
                )
            )
            .into())
        );
    }

    #[test]
    fn compile_record_construction_with_two_elements() {
        assert_eq!(
            compile(
                &RecordConstruction::new(
                    types::Record::new("r", Position::dummy()),
                    vec![
                        RecordElement::new(
                            "x",
                            Number::new(42.0, Position::dummy()),
                            Position::dummy()
                        ),
                        RecordElement::new("y", None::new(Position::dummy()), Position::dummy())
                    ],
                    Position::dummy()
                )
                .into(),
                &TypeContext::dummy(
                    vec![(
                        "r".into(),
                        vec![
                            types::RecordElement::new("x", types::Number::new(Position::dummy())),
                            types::RecordElement::new("y", types::None::new(Position::dummy()))
                        ]
                    )]
                    .into_iter()
                    .collect(),
                    Default::default()
                ),
            ),
            Ok(mir::ir::Let::new(
                "$x",
                mir::types::Type::Number,
                42.0,
                mir::ir::Let::new(
                    "$y",
                    mir::types::Type::None,
                    mir::ir::Expression::None,
                    mir::ir::Record::new(
                        mir::types::Record::new("r"),
                        vec![
                            mir::ir::Variable::new("$x").into(),
                            mir::ir::Variable::new("$y").into()
                        ]
                    )
                )
            )
            .into())
        );
    }

    #[test]
    fn compile_singleton_record_construction() {
        assert_eq!(
            compile(
                &RecordConstruction::new(
                    types::Record::new("r", Position::dummy()),
                    vec![],
                    Position::dummy()
                )
                .into(),
                &TypeContext::dummy(
                    vec![("r".into(), vec![])].into_iter().collect(),
                    Default::default()
                ),
            ),
            Ok(mir::ir::Record::new(mir::types::Record::new("r"), vec![]).into())
        );
    }

    #[test]
    fn compile_record_construction_with_reference_type() {
        assert_eq!(
            compile(
                &RecordConstruction::new(
                    types::Reference::new("r", Position::dummy()),
                    vec![],
                    Position::dummy()
                )
                .into(),
                &TypeContext::dummy(
                    vec![("r".into(), vec![])].into_iter().collect(),
                    vec![(
                        "r".into(),
                        types::Record::new("r", Position::dummy()).into()
                    )]
                    .into_iter()
                    .collect(),
                ),
            ),
            Ok(mir::ir::Record::new(mir::types::Record::new("r"), vec![]).into())
        );
    }
}
