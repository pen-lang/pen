use super::{
    transformation::{
        boolean_operation_transformer, equal_operation_transformer, if_list_transformer,
        not_equal_operation_transformer,
    },
    type_compiler,
    type_context::TypeContext,
    CompileError,
};
use crate::{
    concurrency_configuration::{ConcurrencyConfiguration, MODULE_LOCAL_SPAWN_FUNCTION_NAME},
    transformation::{list_literal_transformer, record_update_transformer},
};
use hir::{
    analysis::types::{
        record_field_resolver, type_canonicalizer, type_equality_checker,
        union_type_member_calculator,
    },
    ir::*,
    types::{self, Type},
};
use std::collections::BTreeMap;

const CLOSURE_NAME: &str = "$closure";
const THUNK_NAME: &str = "$thunk";

pub fn compile(
    expression: &Expression,
    type_context: &TypeContext,
    concurrency_configuration: &ConcurrencyConfiguration,
) -> Result<mir::ir::Expression, CompileError> {
    let compile = |expression| compile(expression, type_context, concurrency_configuration);

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
        Expression::IfList(if_) => compile(&if_list_transformer::transform(if_, type_context)?)?,
        Expression::IfType(if_) => mir::ir::Case::new(
            compile(if_.argument())?,
            if_.branches()
                .iter()
                .map(|branch| {
                    compile_alternatives(
                        if_.name(),
                        branch.type_(),
                        branch.expression(),
                        type_context,
                        concurrency_configuration,
                    )
                })
                .collect::<Result<Vec<_>, CompileError>>()?
                .into_iter()
                .flatten()
                .chain(if let Some(branch) = if_.else_() {
                    if !type_equality_checker::check(
                        branch.type_().unwrap(),
                        &types::Any::new(if_.position().clone()).into(),
                        type_context.types(),
                    )? {
                        compile_alternatives(
                            if_.name(),
                            branch.type_().unwrap(),
                            branch.expression(),
                            type_context,
                            concurrency_configuration,
                        )?
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                })
                .collect(),
            if let Some(branch) = if_.else_() {
                if type_equality_checker::check(
                    branch.type_().unwrap(),
                    &types::Any::new(if_.position().clone()).into(),
                    type_context.types(),
                )? {
                    Some(mir::ir::DefaultAlternative::new(
                        if_.name(),
                        compile(branch.expression())?,
                    ))
                } else {
                    None
                }
            } else {
                None
            },
        )
        .into(),
        Expression::Lambda(lambda) => {
            compile_lambda(lambda, type_context, concurrency_configuration)?
        }
        Expression::Let(let_) => mir::ir::Let::new(
            let_.name().unwrap_or_default(),
            type_compiler::compile(
                let_.type_()
                    .ok_or_else(|| CompileError::TypeNotInferred(let_.position().clone()))?,
                type_context,
            )?,
            compile(let_.bound_expression())?,
            compile(let_.expression())?,
        )
        .into(),
        Expression::List(list) => compile(&list_literal_transformer::transform(
            list,
            type_context.list_type_configuration(),
        ))?,
        Expression::None(_) => mir::ir::Expression::None,
        Expression::Number(number) => mir::ir::Expression::Number(number.value()),
        Expression::Operation(operation) => {
            compile_operation(operation, type_context, concurrency_configuration)?
        }
        Expression::RecordConstruction(construction) => {
            let field_types = record_field_resolver::resolve(
                construction.type_(),
                construction.position(),
                type_context.types(),
                type_context.records(),
            )?;
            let record_type = type_compiler::compile(construction.type_(), type_context)?
                .into_record()
                .unwrap();

            compile_record_fields(
                construction.fields(),
                field_types,
                &|fields| {
                    mir::ir::Record::new(
                        record_type.clone(),
                        field_types
                            .iter()
                            .map(|field_type| fields[field_type.name()].clone())
                            .collect(),
                    )
                    .into()
                },
                type_context,
                concurrency_configuration,
            )?
        }
        Expression::RecordDeconstruction(deconstruction) => {
            let type_ = deconstruction.type_().unwrap();

            mir::ir::RecordField::new(
                type_compiler::compile(type_, type_context)?
                    .into_record()
                    .unwrap(),
                record_field_resolver::resolve(
                    type_,
                    deconstruction.position(),
                    type_context.types(),
                    type_context.records(),
                )?
                .iter()
                .position(|field_type| field_type.name() == deconstruction.field_name())
                .unwrap(),
                compile(deconstruction.record())?,
            )
            .into()
        }
        Expression::RecordUpdate(update) => {
            compile(&record_update_transformer::transform(update, type_context)?)?
        }
        Expression::String(string) => mir::ir::ByteString::new(string.value()).into(),
        Expression::Thunk(thunk) => mir::ir::LetRecursive::new(
            mir::ir::Definition::thunk(
                THUNK_NAME,
                vec![],
                compile(thunk.expression())?,
                type_compiler::compile(
                    thunk
                        .type_()
                        .ok_or_else(|| CompileError::TypeNotInferred(thunk.position().clone()))?,
                    type_context,
                )?,
            ),
            mir::ir::Variable::new(THUNK_NAME),
        )
        .into(),
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
                    Type::Function(function_type) => {
                        let concrete_type = type_compiler::compile_concrete_function(
                            function_type,
                            type_context.types(),
                        )?;

                        mir::ir::Variant::new(
                            concrete_type.clone(),
                            mir::ir::Record::new(concrete_type, vec![argument]),
                        )
                        .into()
                    }
                    Type::List(list_type) => {
                        let concrete_type =
                            type_compiler::compile_concrete_list(list_type, type_context.types())?;

                        mir::ir::Variant::new(
                            concrete_type.clone(),
                            mir::ir::Record::new(concrete_type, vec![argument]),
                        )
                        .into()
                    }
                    Type::Any(_) | Type::Union(_) => argument,
                    Type::Reference(_) => unreachable!(),
                }
            }
        }
        Expression::Variable(variable) => mir::ir::Variable::new(variable.name()).into(),
    })
}

fn compile_lambda(
    lambda: &hir::ir::Lambda,
    type_context: &TypeContext,
    concurrency_configuration: &ConcurrencyConfiguration,
) -> Result<mir::ir::Expression, CompileError> {
    Ok(mir::ir::LetRecursive::new(
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
            compile(lambda.body(), type_context, concurrency_configuration)?,
            type_compiler::compile(lambda.result_type(), type_context)?,
        ),
        mir::ir::Variable::new(CLOSURE_NAME),
    )
    .into())
}

fn compile_alternatives(
    name: &str,
    type_: &Type,
    expression: &Expression,
    type_context: &TypeContext,
    concurrency_configuration: &ConcurrencyConfiguration,
) -> Result<Vec<mir::ir::Alternative>, CompileError> {
    let type_ = type_canonicalizer::canonicalize(type_, type_context.types())?;
    let expression = compile(expression, type_context, concurrency_configuration)?;

    union_type_member_calculator::calculate(&type_, type_context.types())?
        .into_iter()
        .map(|member_type| {
            let compiled_member_type = type_compiler::compile(&member_type, type_context)?;

            Ok(match &member_type {
                Type::Function(function_type) => compile_generic_type_alternative(
                    name,
                    &expression,
                    &type_,
                    &compiled_member_type,
                    &type_compiler::compile_concrete_function(function_type, type_context.types())?,
                ),
                Type::List(list_type) => compile_generic_type_alternative(
                    name,
                    &expression,
                    &type_,
                    &compiled_member_type,
                    &type_compiler::compile_concrete_list(list_type, type_context.types())?,
                ),
                _ => mir::ir::Alternative::new(compiled_member_type.clone(), name, {
                    if type_.is_union() {
                        mir::ir::Let::new(
                            name,
                            mir::types::Type::Variant,
                            mir::ir::Variant::new(
                                compiled_member_type,
                                mir::ir::Variable::new(name),
                            ),
                            expression.clone(),
                        )
                        .into()
                    } else {
                        expression.clone()
                    }
                }),
            })
        })
        .collect::<Result<_, _>>()
}

fn compile_generic_type_alternative(
    name: &str,
    expression: &mir::ir::Expression,
    type_: &hir::types::Type,
    member_type: &mir::types::Type,
    concrete_member_type: &mir::types::Record,
) -> mir::ir::Alternative {
    mir::ir::Alternative::new(concrete_member_type.clone(), name, {
        if type_.is_union() {
            mir::ir::Let::new(
                name,
                mir::types::Type::Variant,
                mir::ir::Variant::new(concrete_member_type.clone(), mir::ir::Variable::new(name)),
                expression.clone(),
            )
        } else {
            mir::ir::Let::new(
                name,
                member_type.clone(),
                mir::ir::RecordField::new(
                    concrete_member_type.clone(),
                    0,
                    mir::ir::Variable::new(name),
                ),
                expression.clone(),
            )
        }
    })
}

fn compile_operation(
    operation: &Operation,
    type_context: &TypeContext,
    concurrency_configuration: &ConcurrencyConfiguration,
) -> Result<mir::ir::Expression, CompileError> {
    let compile = |expression| compile(expression, type_context, concurrency_configuration);

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
        Operation::Spawn(operation) => {
            let thunk_type = mir::types::Function::new(vec![], mir::types::Type::Variant);

            // TODO Downcast outputs.
            mir::ir::Call::new(
                mir::types::Function::new(vec![thunk_type.clone().into()], thunk_type),
                mir::ir::Variable::new(MODULE_LOCAL_SPAWN_FUNCTION_NAME),
                vec![mir::ir::LetRecursive::new(
                    mir::ir::Definition::thunk(
                        THUNK_NAME,
                        vec![],
                        // TODO Upcast inputs.
                        compile(operation.function().body())?,
                        type_compiler::compile(operation.function().result_type(), type_context)?,
                    ),
                    mir::ir::Variable::new(THUNK_NAME),
                )
                .into()],
            )
            .into()
        }
        Operation::Boolean(operation) => {
            compile(&boolean_operation_transformer::transform(operation))?
        }
        Operation::Equality(operation) => match operation.operator() {
            EqualityOperator::Equal => {
                match type_canonicalizer::canonicalize(
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
                    _ => compile(&equal_operation_transformer::transform(
                        operation,
                        type_context,
                    )?)?,
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
        Operation::Try(operation) => {
            let success_type = operation
                .type_()
                .ok_or_else(|| CompileError::TypeNotInferred(operation.position().clone()))?;
            let error_type = type_compiler::compile(
                &types::Reference::new(
                    &type_context.error_type_configuration().error_type_name,
                    operation.position().clone(),
                )
                .into(),
                type_context,
            )?;

            mir::ir::Case::new(
                mir::ir::TryOperation::new(
                    compile(operation.expression())?,
                    "$error",
                    error_type.clone(),
                    mir::ir::Variant::new(error_type, mir::ir::Variable::new("$error")),
                ),
                compile_alternatives(
                    "$success",
                    success_type,
                    &Variable::new("$success", operation.position().clone()).into(),
                    type_context,
                    concurrency_configuration,
                )?,
                None,
            )
            .into()
        }
    })
}

fn compile_record_fields(
    fields: &[RecordField],
    field_types: &[types::RecordField],
    convert_fields_to_expression: &dyn Fn(
        &BTreeMap<String, mir::ir::Expression>,
    ) -> mir::ir::Expression,
    type_context: &TypeContext,
    concurrency_configuration: &ConcurrencyConfiguration,
) -> Result<mir::ir::Expression, CompileError> {
    Ok(match fields {
        [] => convert_fields_to_expression(&Default::default()),
        [field, ..] => {
            let field_name = format!("${}", field.name());

            mir::ir::Let::new(
                field_name.clone(),
                type_compiler::compile(
                    field_types
                        .iter()
                        .find(|field_type| field_type.name() == field.name())
                        .ok_or_else(|| CompileError::RecordFieldUnknown(field.position().clone()))?
                        .type_(),
                    type_context,
                )?,
                compile(field.expression(), type_context, concurrency_configuration)?,
                compile_record_fields(
                    &fields[1..],
                    field_types,
                    &|fields| {
                        convert_fields_to_expression(
                            &fields
                                .clone()
                                .into_iter()
                                .chain(vec![(
                                    field.name().into(),
                                    mir::ir::Variable::new(field_name.clone()).into(),
                                )])
                                .collect(),
                        )
                    },
                    type_context,
                    concurrency_configuration,
                )?,
            )
            .into()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::concurrency_configuration::CONCURRENCY_CONFIGURATION;
    use position::{test::PositionFake, Position};

    mod if_type {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn compile_with_union() {
            assert_eq!(
                compile(
                    &IfType::new(
                        "y",
                        Variable::new("x", Position::fake()),
                        vec![
                            IfTypeBranch::new(
                                types::Number::new(Position::fake()),
                                None::new(Position::fake()),
                            ),
                            IfTypeBranch::new(
                                types::None::new(Position::fake()),
                                None::new(Position::fake()),
                            )
                        ],
                        None,
                        Position::fake(),
                    )
                    .into(),
                    &TypeContext::dummy(Default::default(), Default::default()),
                    &CONCURRENCY_CONFIGURATION,
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::Variable::new("x"),
                    vec![
                        mir::ir::Alternative::new(
                            mir::types::Type::Number,
                            "y",
                            mir::ir::Expression::None
                        ),
                        mir::ir::Alternative::new(
                            mir::types::Type::None,
                            "y",
                            mir::ir::Expression::None
                        )
                    ],
                    None
                )
                .into())
            );
        }

        #[test]
        fn compile_with_union_and_else() {
            assert_eq!(
                compile(
                    &IfType::new(
                        "y",
                        Variable::new("x", Position::fake()),
                        vec![IfTypeBranch::new(
                            types::Number::new(Position::fake()),
                            None::new(Position::fake()),
                        )],
                        Some(ElseBranch::new(
                            Some(types::None::new(Position::fake()).into()),
                            None::new(Position::fake()),
                            Position::fake()
                        )),
                        Position::fake(),
                    )
                    .into(),
                    &TypeContext::dummy(Default::default(), Default::default()),
                    &CONCURRENCY_CONFIGURATION,
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::Variable::new("x"),
                    vec![
                        mir::ir::Alternative::new(
                            mir::types::Type::Number,
                            "y",
                            mir::ir::Expression::None
                        ),
                        mir::ir::Alternative::new(
                            mir::types::Type::None,
                            "y",
                            mir::ir::Expression::None
                        )
                    ],
                    None
                )
                .into())
            );
        }

        #[test]
        fn compile_with_any() {
            assert_eq!(
                compile(
                    &IfType::new(
                        "y",
                        Variable::new("x", Position::fake()),
                        vec![IfTypeBranch::new(
                            types::Number::new(Position::fake()),
                            None::new(Position::fake()),
                        )],
                        Some(ElseBranch::new(
                            Some(types::Any::new(Position::fake()).into()),
                            None::new(Position::fake()),
                            Position::fake()
                        )),
                        Position::fake(),
                    )
                    .into(),
                    &TypeContext::dummy(Default::default(), Default::default()),
                    &CONCURRENCY_CONFIGURATION,
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::Variable::new("x"),
                    vec![mir::ir::Alternative::new(
                        mir::types::Type::Number,
                        "y",
                        mir::ir::Expression::None
                    )],
                    Some(mir::ir::DefaultAlternative::new(
                        "y",
                        mir::ir::Expression::None
                    ))
                )
                .into())
            );
        }

        #[test]
        fn compile_with_union_branch() {
            assert_eq!(
                compile(
                    &IfType::new(
                        "y",
                        Variable::new("x", Position::fake()),
                        vec![IfTypeBranch::new(
                            types::Union::new(
                                types::Number::new(Position::fake()),
                                types::None::new(Position::fake()),
                                Position::fake()
                            ),
                            None::new(Position::fake()),
                        )],
                        None,
                        Position::fake(),
                    )
                    .into(),
                    &TypeContext::dummy(Default::default(), Default::default()),
                    &CONCURRENCY_CONFIGURATION,
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::Variable::new("x"),
                    vec![
                        mir::ir::Alternative::new(
                            mir::types::Type::None,
                            "y",
                            mir::ir::Let::new(
                                "y",
                                mir::types::Type::Variant,
                                mir::ir::Variant::new(
                                    mir::types::Type::None,
                                    mir::ir::Variable::new("y")
                                ),
                                mir::ir::Expression::None
                            )
                        ),
                        mir::ir::Alternative::new(
                            mir::types::Type::Number,
                            "y",
                            mir::ir::Let::new(
                                "y",
                                mir::types::Type::Variant,
                                mir::ir::Variant::new(
                                    mir::types::Type::Number,
                                    mir::ir::Variable::new("y")
                                ),
                                mir::ir::Expression::None
                            )
                        ),
                    ],
                    None
                )
                .into())
            );
        }

        #[test]
        fn compile_function_branch() {
            let type_context = TypeContext::dummy(Default::default(), Default::default());
            let function_type =
                types::Function::new(vec![], types::None::new(Position::fake()), Position::fake());
            let concrete_function_type =
                type_compiler::compile_concrete_function(&function_type, type_context.types())
                    .unwrap();

            assert_eq!(
                compile(
                    &IfType::new(
                        "y",
                        Variable::new("x", Position::fake()),
                        vec![IfTypeBranch::new(
                            function_type.clone(),
                            Variable::new("y", Position::fake()),
                        )],
                        None,
                        Position::fake(),
                    )
                    .into(),
                    &type_context,
                    &CONCURRENCY_CONFIGURATION,
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::Variable::new("x"),
                    vec![mir::ir::Alternative::new(
                        concrete_function_type.clone(),
                        "y",
                        mir::ir::Let::new(
                            "y",
                            type_compiler::compile_function(&function_type, &type_context).unwrap(),
                            mir::ir::RecordField::new(
                                concrete_function_type,
                                0,
                                mir::ir::Variable::new("y")
                            ),
                            mir::ir::Variable::new("y")
                        ),
                    )],
                    None
                )
                .into())
            );
        }

        #[test]
        fn compile_list_branch() {
            let type_context = TypeContext::dummy(Default::default(), Default::default());
            let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());
            let concrete_list_type =
                type_compiler::compile_concrete_list(&list_type, type_context.types()).unwrap();

            assert_eq!(
                compile(
                    &IfType::new(
                        "y",
                        Variable::new("x", Position::fake()),
                        vec![IfTypeBranch::new(
                            list_type,
                            Variable::new("y", Position::fake()),
                        )],
                        None,
                        Position::fake(),
                    )
                    .into(),
                    &type_context,
                    &CONCURRENCY_CONFIGURATION,
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::Variable::new("x"),
                    vec![mir::ir::Alternative::new(
                        concrete_list_type.clone(),
                        "y",
                        mir::ir::Let::new(
                            "y",
                            mir::types::Record::new(
                                &type_context.list_type_configuration().list_type_name
                            ),
                            mir::ir::RecordField::new(
                                concrete_list_type,
                                0,
                                mir::ir::Variable::new("y")
                            ),
                            mir::ir::Variable::new("y")
                        ),
                    )],
                    None
                )
                .into())
            );
        }

        #[test]
        fn compile_union_branch_including_list() {
            let type_context = TypeContext::dummy(Default::default(), Default::default());
            let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());
            let concrete_list_type =
                type_compiler::compile_concrete_list(&list_type, type_context.types()).unwrap();

            assert_eq!(
                compile(
                    &IfType::new(
                        "y",
                        Variable::new("x", Position::fake()),
                        vec![IfTypeBranch::new(
                            types::Union::new(
                                list_type,
                                types::None::new(Position::fake()),
                                Position::fake()
                            ),
                            Variable::new("y", Position::fake()),
                        )],
                        None,
                        Position::fake(),
                    )
                    .into(),
                    &type_context,
                    &CONCURRENCY_CONFIGURATION,
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::Variable::new("x"),
                    vec![
                        mir::ir::Alternative::new(
                            concrete_list_type.clone(),
                            "y",
                            mir::ir::Let::new(
                                "y",
                                mir::types::Type::Variant,
                                mir::ir::Variant::new(
                                    concrete_list_type,
                                    mir::ir::Variable::new("y")
                                ),
                                mir::ir::Variable::new("y")
                            ),
                        ),
                        mir::ir::Alternative::new(
                            mir::types::Type::None,
                            "y",
                            mir::ir::Let::new(
                                "y",
                                mir::types::Type::Variant,
                                mir::ir::Variant::new(
                                    mir::types::Type::None,
                                    mir::ir::Variable::new("y")
                                ),
                                mir::ir::Variable::new("y")
                            ),
                        )
                    ],
                    None
                )
                .into())
            );
        }
    }

    mod records {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn compile_record_construction() {
            assert_eq!(
                compile(
                    &RecordConstruction::new(
                        types::Record::new("r", Position::fake()),
                        vec![RecordField::new(
                            "x",
                            None::new(Position::fake()),
                            Position::fake()
                        )],
                        Position::fake()
                    )
                    .into(),
                    &TypeContext::dummy(
                        Default::default(),
                        vec![(
                            "r".into(),
                            vec![types::RecordField::new(
                                "x",
                                types::None::new(Position::fake())
                            )]
                        )]
                        .into_iter()
                        .collect()
                    ),
                    &CONCURRENCY_CONFIGURATION,
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
        fn compile_record_construction_with_two_fields() {
            assert_eq!(
                compile(
                    &RecordConstruction::new(
                        types::Record::new("r", Position::fake()),
                        vec![
                            RecordField::new(
                                "x",
                                Number::new(42.0, Position::fake()),
                                Position::fake()
                            ),
                            RecordField::new("y", None::new(Position::fake()), Position::fake())
                        ],
                        Position::fake()
                    )
                    .into(),
                    &TypeContext::dummy(
                        Default::default(),
                        vec![(
                            "r".into(),
                            vec![
                                types::RecordField::new("x", types::Number::new(Position::fake())),
                                types::RecordField::new("y", types::None::new(Position::fake()))
                            ]
                        )]
                        .into_iter()
                        .collect()
                    ),
                    &CONCURRENCY_CONFIGURATION,
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
                        types::Record::new("r", Position::fake()),
                        vec![],
                        Position::fake()
                    )
                    .into(),
                    &TypeContext::dummy(
                        Default::default(),
                        vec![("r".into(), vec![])].into_iter().collect()
                    ),
                    &CONCURRENCY_CONFIGURATION,
                ),
                Ok(mir::ir::Record::new(mir::types::Record::new("r"), vec![]).into())
            );
        }

        #[test]
        fn compile_record_construction_with_reference_type() {
            assert_eq!(
                compile(
                    &RecordConstruction::new(
                        types::Reference::new("r", Position::fake()),
                        vec![],
                        Position::fake()
                    )
                    .into(),
                    &TypeContext::dummy(
                        vec![("r".into(), types::Record::new("r", Position::fake()).into())]
                            .into_iter()
                            .collect(),
                        vec![("r".into(), vec![])].into_iter().collect()
                    ),
                    &CONCURRENCY_CONFIGURATION,
                ),
                Ok(mir::ir::Record::new(mir::types::Record::new("r"), vec![]).into())
            );
        }
    }

    mod try_operation {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn compile_with_none() {
            let error_type = mir::types::Record::new("error");

            assert_eq!(
                compile(
                    &TryOperation::new(
                        Some(types::None::new(Position::fake()).into()),
                        Variable::new("x", Position::fake()),
                        Position::fake(),
                    )
                    .into(),
                    &TypeContext::dummy(
                        vec![(
                            "error".into(),
                            types::Record::new("error", Position::fake()).into()
                        )]
                        .into_iter()
                        .collect(),
                        Default::default()
                    ),
                    &CONCURRENCY_CONFIGURATION,
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::TryOperation::new(
                        mir::ir::Variable::new("x"),
                        "$error",
                        error_type.clone(),
                        mir::ir::Variant::new(error_type, mir::ir::Variable::new("$error"))
                    ),
                    vec![mir::ir::Alternative::new(
                        mir::types::Type::None,
                        "$success",
                        mir::ir::Variable::new("$success"),
                    )],
                    None
                )
                .into())
            );
        }

        #[test]
        fn compile_with_union() {
            let error_type = mir::types::Record::new("error");

            assert_eq!(
                compile(
                    &TryOperation::new(
                        Some(
                            types::Union::new(
                                types::Number::new(Position::fake()),
                                types::None::new(Position::fake()),
                                Position::fake()
                            )
                            .into()
                        ),
                        Variable::new("x", Position::fake()),
                        Position::fake(),
                    )
                    .into(),
                    &TypeContext::dummy(
                        vec![(
                            "error".into(),
                            types::Record::new("error", Position::fake()).into()
                        )]
                        .into_iter()
                        .collect(),
                        Default::default()
                    ),
                    &CONCURRENCY_CONFIGURATION,
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::TryOperation::new(
                        mir::ir::Variable::new("x"),
                        "$error",
                        error_type.clone(),
                        mir::ir::Variant::new(error_type, mir::ir::Variable::new("$error"))
                    ),
                    vec![
                        mir::ir::Alternative::new(
                            mir::types::Type::None,
                            "$success",
                            mir::ir::Let::new(
                                "$success",
                                mir::types::Type::Variant,
                                mir::ir::Variant::new(
                                    mir::types::Type::None,
                                    mir::ir::Variable::new("$success")
                                ),
                                mir::ir::Variable::new("$success"),
                            ),
                        ),
                        mir::ir::Alternative::new(
                            mir::types::Type::Number,
                            "$success",
                            mir::ir::Let::new(
                                "$success",
                                mir::types::Type::Variant,
                                mir::ir::Variant::new(
                                    mir::types::Type::Number,
                                    mir::ir::Variable::new("$success")
                                ),
                                mir::ir::Variable::new("$success"),
                            ),
                        )
                    ],
                    None
                )
                .into())
            );
        }
    }
}
