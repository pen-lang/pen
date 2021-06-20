use super::{
    transformation::{
        boolean_operation_transformer, equal_operation_transformer, not_equal_operation_transformer,
    },
    type_compiler,
    type_compiler::NONE_RECORD_TYPE_NAME,
    type_context::TypeContext,
    CompileError,
};
use crate::{
    hir::*,
    types::{
        analysis::{type_canonicalizer, type_resolver},
        Type,
    },
};

const CLOSURE_NAME: &str = "$closure";
const UNUSED_VARIABLE: &str = "$unused";

pub fn compile(
    expression: &Expression,
    type_context: &TypeContext,
) -> Result<mir::ir::Expression, CompileError> {
    let compile = |expression| compile(expression, type_context);

    Ok(match expression {
        Expression::Boolean(boolean) => mir::ir::Expression::Boolean(boolean.value()),
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
                compile_block(lambda.body(), type_context)?,
                type_compiler::compile(lambda.result_type(), type_context)?,
            ),
            mir::ir::Variable::new(CLOSURE_NAME),
        )
        .into(),
        Expression::None(_) => {
            mir::ir::Record::new(mir::types::Record::new(NONE_RECORD_TYPE_NAME), vec![]).into()
        }
        Expression::Number(number) => mir::ir::Expression::Number(number.value()),
        Expression::Operation(operation) => compile_operation(operation, type_context)?,
        Expression::String(string) => mir::ir::ByteString::new(string.value()).into(),
        Expression::TypeCoercion(coercion) => {
            let from = type_canonicalizer::canonicalize(coercion.from(), type_context.types())?;
            let to = type_canonicalizer::canonicalize(coercion.to(), type_context.types())?;

            if from.is_list() && to.is_list() {
                compile(coercion.argument())?
            } else {
                // Coerce to union or Any types.
                let argument = compile(coercion.argument())?;

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
        Operation::Not(_) => todo!(),
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

pub fn compile_block(
    block: &Block,
    type_context: &TypeContext,
) -> Result<mir::ir::Expression, CompileError> {
    let mut expression = compile(block.expression(), type_context)?;

    for statement in block.statements().iter().rev() {
        expression = mir::ir::Let::new(
            statement.name().unwrap_or(UNUSED_VARIABLE),
            type_compiler::compile(
                statement
                    .type_()
                    .ok_or_else(|| CompileError::TypeNotInferred(statement.position().clone()))?,
                type_context,
            )?,
            compile(statement.expression(), type_context)?,
            expression,
        )
        .into();
    }

    Ok(expression)
}
