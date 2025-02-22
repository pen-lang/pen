use super::{
    CompileError, built_in_call,
    context::Context,
    transformation::{
        boolean_operation, equal_operation, if_list, if_map, list_literal, map_literal,
        not_equal_operation,
    },
    type_,
};
use crate::{concrete_type, list_comprehension};
use fnv::FnvHashMap;
use hir::{
    analysis::{
        AnalysisError, record_field_resolver, type_canonicalizer, type_equality_checker,
        union_type_member_calculator,
    },
    ir::*,
    types::{self, Type},
};

pub fn compile(
    context: &Context,
    expression: &Expression,
) -> Result<mir::ir::Expression, CompileError> {
    let compile = |expression| compile(context, expression);

    Ok(match expression {
        Expression::Boolean(boolean) => mir::ir::Expression::Boolean(boolean.value()),
        Expression::BuiltInFunction(function) => {
            return Err(
                AnalysisError::BuiltInFunctionNotCalled(function.position().clone()).into(),
            );
        }
        Expression::Call(call) => {
            if let Expression::BuiltInFunction(function) = call.function() {
                built_in_call::compile(context, call, function)?
            } else {
                let type_ = call
                    .function_type()
                    .ok_or_else(|| AnalysisError::TypeNotInferred(call.position().clone()))?;

                mir::ir::Call::new(
                    type_::compile(context, type_)?
                        .into_function()
                        .ok_or_else(|| {
                            AnalysisError::FunctionExpected(
                                call.function().position().clone(),
                                type_.clone(),
                            )
                        })?,
                    compile(call.function())?,
                    call.arguments()
                        .iter()
                        .map(compile)
                        .collect::<Result<_, _>>()?,
                )
                .into()
            }
        }
        Expression::If(if_) => mir::ir::If::new(
            compile(if_.condition())?,
            compile(if_.then())?,
            compile(if_.else_())?,
        )
        .into(),
        Expression::IfList(if_) => compile(&if_list::transform(context, if_)?)?,
        Expression::IfMap(if_) => compile(&if_map::transform(context, if_)?)?,
        Expression::IfType(if_) => mir::ir::Case::new(
            compile(if_.argument())?,
            if_.branches()
                .iter()
                .map(|branch| {
                    compile_alternative(context, if_.name(), branch.type_(), branch.expression())
                })
                .chain(if let Some(branch) = if_.else_() {
                    if !type_equality_checker::check(
                        branch.type_().unwrap(),
                        &types::Any::new(if_.position().clone()).into(),
                        context.types(),
                    )? {
                        Some(Ok(compile_alternative(
                            context,
                            if_.name(),
                            branch.type_().unwrap(),
                            branch.expression(),
                        )?))
                    } else {
                        None
                    }
                } else {
                    None
                })
                .collect::<Result<_, _>>()?,
            if let Some(branch) = if_.else_() {
                if type_equality_checker::check(
                    branch.type_().unwrap(),
                    &types::Any::new(if_.position().clone()).into(),
                    context.types(),
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
        Expression::Lambda(lambda) => compile_lambda(context, lambda)?,
        Expression::Let(let_) => mir::ir::Let::new(
            let_.name().unwrap_or_default(),
            type_::compile(
                context,
                let_.type_()
                    .ok_or_else(|| AnalysisError::TypeNotInferred(let_.position().clone()))?,
            )?,
            compile(let_.bound_expression())?,
            compile(let_.expression())?,
        )
        .into(),
        Expression::List(list) => compile(&list_literal::transform(context, list)?)?,
        Expression::ListComprehension(comprehension) => {
            list_comprehension::compile(context, comprehension)?
        }
        Expression::Map(map) => compile(&map_literal::transform(context, map)?)?,
        Expression::None(_) => mir::ir::Expression::None,
        Expression::Number(number) => mir::ir::Expression::Number(number.value()),
        Expression::Operation(operation) => compile_operation(context, operation)?,
        Expression::RecordConstruction(construction) => {
            let field_types = record_field_resolver::resolve(
                construction.type_(),
                construction.type_().position(),
                context.types(),
                context.records(),
            )?;
            let record_type = type_::compile(context, construction.type_())?
                .into_record()
                .unwrap();

            compile_record_fields(context, construction.fields(), field_types, &|fields| {
                mir::ir::Record::new(
                    record_type.clone(),
                    field_types
                        .iter()
                        .map(|field_type| fields[field_type.name()].clone())
                        .collect(),
                )
                .into()
            })?
        }
        Expression::RecordDeconstruction(deconstruction) => {
            let type_ = deconstruction.type_().unwrap();

            mir::ir::RecordField::new(
                type_::compile(context, type_)?.into_record().unwrap(),
                record_field_resolver::resolve(
                    type_,
                    deconstruction.record().position(),
                    context.types(),
                    context.records(),
                )?
                .iter()
                .position(|field_type| field_type.name() == deconstruction.field_name())
                .unwrap(),
                compile(deconstruction.record())?,
            )
            .into()
        }
        Expression::RecordUpdate(update) => mir::ir::RecordUpdate::new(
            type_::compile(context, update.type_())?
                .into_record()
                .unwrap(),
            compile(update.record())?,
            update
                .fields()
                .iter()
                .map(|field| -> Result<_, CompileError> {
                    Ok(mir::ir::RecordUpdateField::new(
                        record_field_resolver::resolve(
                            update.type_(),
                            update.type_().position(),
                            context.types(),
                            context.records(),
                        )?
                        .iter()
                        .position(|field_type| field_type.name() == field.name())
                        .unwrap(),
                        compile(field.expression())?,
                    ))
                })
                .collect::<Result<_, _>>()?,
        )
        .into(),
        Expression::String(string) => mir::ir::ByteString::new(string.value()).into(),
        Expression::Thunk(thunk) => {
            const THUNK_NAME: &str = "$thunk";

            mir::ir::LetRecursive::new(
                mir::ir::FunctionDefinition::thunk(
                    THUNK_NAME,
                    type_::compile(
                        context,
                        thunk.type_().ok_or_else(|| {
                            AnalysisError::TypeNotInferred(thunk.position().clone())
                        })?,
                    )?,
                    compile(thunk.expression())?,
                ),
                mir::ir::Variable::new(THUNK_NAME),
            )
            .into()
        }
        Expression::TypeCoercion(coercion) => {
            let from = type_canonicalizer::canonicalize(coercion.from(), context.types())?;
            let to = type_canonicalizer::canonicalize(coercion.to(), context.types())?;
            let argument = compile(coercion.argument())?;

            match &from {
                Type::Boolean(_)
                | Type::Error(_)
                | Type::None(_)
                | Type::Number(_)
                | Type::Record(_)
                | Type::String(_) => {
                    mir::ir::Variant::new(type_::compile(context, &from)?, argument).into()
                }
                Type::Function(function_type) => mir::ir::Variant::new(
                    type_::compile_concrete_function(function_type, context.types())?,
                    concrete_type::compile(context, argument, &from)?,
                )
                .into(),
                Type::List(list_type) => {
                    if to.is_list() {
                        argument
                    } else {
                        mir::ir::Variant::new(
                            type_::compile_concrete_list(list_type, context.types())?,
                            concrete_type::compile(context, argument, &from)?,
                        )
                        .into()
                    }
                }
                Type::Map(map_type) => {
                    if to.is_map() {
                        argument
                    } else {
                        mir::ir::Variant::new(
                            type_::compile_concrete_map(map_type, context.types())?,
                            concrete_type::compile(context, argument, &from)?,
                        )
                        .into()
                    }
                }
                Type::Any(_) | Type::Union(_) => argument,
                Type::Reference(_) => unreachable!(),
            }
        }
        Expression::Variable(variable) => mir::ir::Variable::new(variable.name()).into(),
    })
}

fn compile_lambda(
    context: &Context,
    lambda: &hir::ir::Lambda,
) -> Result<mir::ir::Expression, CompileError> {
    const CLOSURE_NAME: &str = "$closure";

    Ok(mir::ir::LetRecursive::new(
        mir::ir::FunctionDefinition::new(
            CLOSURE_NAME,
            lambda
                .arguments()
                .iter()
                .map(|argument| -> Result<_, CompileError> {
                    Ok(mir::ir::Argument::new(
                        argument.name(),
                        type_::compile(context, argument.type_())?,
                    ))
                })
                .collect::<Result<_, _>>()?,
            type_::compile(context, lambda.result_type())?,
            compile(context, lambda.body())?,
        ),
        mir::ir::Variable::new(CLOSURE_NAME),
    )
    .into())
}

fn compile_alternative(
    context: &Context,
    name: &str,
    type_: &Type,
    expression: &Expression,
) -> Result<mir::ir::Alternative, CompileError> {
    let type_ = type_canonicalizer::canonicalize(type_, context.types())?;
    let expression = compile(context, expression)?;
    let compile_generic_type_alternative = |generic_type| -> Result<_, CompileError> {
        Ok(compile_generic_type_alternative(
            name,
            &expression,
            &type_::compile(context, &type_)?,
            generic_type,
        ))
    };

    Ok(match &type_ {
        Type::Function(function_type) => compile_generic_type_alternative(
            &type_::compile_concrete_function(function_type, context.types())?,
        )?,
        Type::List(list_type) => compile_generic_type_alternative(&type_::compile_concrete_list(
            list_type,
            context.types(),
        )?)?,
        Type::Map(map_type) => compile_generic_type_alternative(&type_::compile_concrete_map(
            map_type,
            context.types(),
        )?)?,
        _ => mir::ir::Alternative::new(
            union_type_member_calculator::calculate(&type_, context.types())?
                .iter()
                .map(|type_| type_::compile_concrete(context, type_))
                .collect::<Result<_, _>>()?,
            name,
            expression.clone(),
        ),
    })
}

fn compile_generic_type_alternative(
    name: &str,
    expression: &mir::ir::Expression,
    member_type: &mir::types::Type,
    concrete_member_type: &mir::types::Record,
) -> mir::ir::Alternative {
    mir::ir::Alternative::new(vec![concrete_member_type.clone().into()], name, {
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
    })
}

fn compile_operation(
    context: &Context,
    operation: &Operation,
) -> Result<mir::ir::Expression, CompileError> {
    let compile = |expression| compile(context, expression);

    Ok(match operation {
        Operation::Addition(operation) => match operation
            .type_()
            .ok_or_else(|| AnalysisError::TypeNotInferred(operation.position().clone()))?
        {
            Type::Number(_) => mir::ir::ArithmeticOperation::new(
                mir::ir::ArithmeticOperator::Add,
                compile(operation.lhs())?,
                compile(operation.rhs())?,
            )
            .into(),
            Type::String(_) => mir::ir::StringConcatenation::new(vec![
                compile(operation.lhs())?,
                compile(operation.rhs())?,
            ])
            .into(),
            type_ => {
                return Err(AnalysisError::InvalidAdditionOperand(type_.position().clone()).into());
            }
        },
        Operation::Arithmetic(operation) => mir::ir::ArithmeticOperation::new(
            match operation.operator() {
                ArithmeticOperator::Subtract => mir::ir::ArithmeticOperator::Subtract,
                ArithmeticOperator::Multiply => mir::ir::ArithmeticOperator::Multiply,
                ArithmeticOperator::Divide => mir::ir::ArithmeticOperator::Divide,
            },
            compile(operation.lhs())?,
            compile(operation.rhs())?,
        )
        .into(),
        Operation::Boolean(operation) => compile(&boolean_operation::transform(operation))?,
        Operation::Equality(operation) => match operation.operator() {
            EqualityOperator::Equal => {
                match type_canonicalizer::canonicalize(
                    operation.type_().ok_or_else(|| {
                        AnalysisError::TypeNotInferred(operation.position().clone())
                    })?,
                    context.types(),
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
                            &context.configuration()?.string_type.equal_function_name,
                        ),
                        vec![compile(operation.lhs())?, compile(operation.rhs())?],
                    )
                    .into(),
                    _ => compile(&equal_operation::expression::transform(context, operation)?)?,
                }
            }
            EqualityOperator::NotEqual => compile(&not_equal_operation::transform(operation))?,
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
            const SUCCESS_NAME: &str = "$success";
            const ERROR_NAME: &str = "$error";

            let error_type = type_::compile(
                context,
                &types::Error::new(operation.position().clone()).into(),
            )?;

            mir::ir::Case::new(
                mir::ir::TryOperation::new(
                    compile(operation.expression())?,
                    ERROR_NAME,
                    error_type.clone(),
                    mir::ir::Variant::new(error_type, mir::ir::Variable::new(ERROR_NAME)),
                ),
                vec![compile_alternative(
                    context,
                    SUCCESS_NAME,
                    operation.type_().ok_or_else(|| {
                        AnalysisError::TypeNotInferred(operation.position().clone())
                    })?,
                    &Variable::new(SUCCESS_NAME, operation.position().clone()).into(),
                )?],
                None,
            )
            .into()
        }
    })
}

fn compile_record_fields(
    context: &Context,
    fields: &[RecordField],
    field_types: &[types::RecordField],
    convert_fields_to_expression: &dyn Fn(
        &FnvHashMap<String, mir::ir::Expression>,
    ) -> mir::ir::Expression,
) -> Result<mir::ir::Expression, CompileError> {
    Ok(match fields {
        [] => convert_fields_to_expression(&Default::default()),
        [field, ..] => {
            let field_name = format!("${}", field.name());

            mir::ir::Let::new(
                field_name.clone(),
                type_::compile(
                    context,
                    field_types
                        .iter()
                        .find(|field_type| field_type.name() == field.name())
                        .ok_or_else(|| AnalysisError::RecordFieldUnknown(field.position().clone()))?
                        .type_(),
                )?,
                compile(context, field.expression())?,
                compile_record_fields(context, &fields[1..], field_types, &|fields| {
                    convert_fields_to_expression(
                        &fields
                            .clone()
                            .into_iter()
                            .chain([(
                                field.name().into(),
                                mir::ir::Variable::new(field_name.clone()).into(),
                            )])
                            .collect(),
                    )
                })?,
            )
            .into()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use position::{Position, test::PositionFake};

    fn compile_expression(expression: &Expression) -> Result<mir::ir::Expression, CompileError> {
        compile(
            &Context::dummy(Default::default(), Default::default()),
            expression,
        )
    }

    mod if_type {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn compile_with_union() {
            assert_eq!(
                compile_expression(
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
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::Variable::new("x"),
                    vec![
                        mir::ir::Alternative::new(
                            vec![mir::types::Type::Number],
                            "y",
                            mir::ir::Expression::None
                        ),
                        mir::ir::Alternative::new(
                            vec![mir::types::Type::None],
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
                compile_expression(
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
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::Variable::new("x"),
                    vec![
                        mir::ir::Alternative::new(
                            vec![mir::types::Type::Number],
                            "y",
                            mir::ir::Expression::None
                        ),
                        mir::ir::Alternative::new(
                            vec![mir::types::Type::None],
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
                compile_expression(
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
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::Variable::new("x"),
                    vec![mir::ir::Alternative::new(
                        vec![mir::types::Type::Number],
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
                compile_expression(
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
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::Variable::new("x"),
                    vec![mir::ir::Alternative::new(
                        vec![mir::types::Type::None, mir::types::Type::Number],
                        "y",
                        mir::ir::Expression::None
                    )],
                    None
                )
                .into())
            );
        }

        #[test]
        fn compile_function_branch() {
            let context = Context::dummy(Default::default(), Default::default());
            let function_type =
                types::Function::new(vec![], types::None::new(Position::fake()), Position::fake());
            let concrete_function_type =
                type_::compile_concrete_function(&function_type, context.types()).unwrap();

            assert_eq!(
                compile(
                    &context,
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
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::Variable::new("x"),
                    vec![mir::ir::Alternative::new(
                        vec![concrete_function_type.clone().into()],
                        "y",
                        mir::ir::Let::new(
                            "y",
                            type_::compile_function(&context, &function_type).unwrap(),
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
            let context = Context::dummy(Default::default(), Default::default());
            let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());
            let concrete_list_type =
                type_::compile_concrete_list(&list_type, context.types()).unwrap();

            assert_eq!(
                compile(
                    &context,
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
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::Variable::new("x"),
                    vec![mir::ir::Alternative::new(
                        vec![concrete_list_type.clone().into()],
                        "y",
                        mir::ir::Let::new(
                            "y",
                            mir::types::Record::new(
                                &context.configuration().unwrap().list_type.list_type_name
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
            let context = Context::dummy(Default::default(), Default::default());
            let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());
            let concrete_list_type =
                type_::compile_concrete_list(&list_type, context.types()).unwrap();

            assert_eq!(
                compile(
                    &context,
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
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::Variable::new("x"),
                    vec![mir::ir::Alternative::new(
                        vec![concrete_list_type.into(), mir::types::Type::None],
                        "y",
                        mir::ir::Variable::new("y")
                    )],
                    None
                )
                .into())
            );
        }

        #[test]
        fn compile_map_branch() {
            let context = Context::dummy(Default::default(), Default::default());
            let map_type = types::Map::new(
                types::None::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );
            let concrete_map_type =
                type_::compile_concrete_map(&map_type, context.types()).unwrap();

            assert_eq!(
                compile(
                    &context,
                    &IfType::new(
                        "y",
                        Variable::new("x", Position::fake()),
                        vec![IfTypeBranch::new(
                            map_type,
                            Variable::new("y", Position::fake()),
                        )],
                        None,
                        Position::fake(),
                    )
                    .into(),
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::Variable::new("x"),
                    vec![mir::ir::Alternative::new(
                        vec![concrete_map_type.clone().into()],
                        "y",
                        mir::ir::Let::new(
                            "y",
                            mir::types::Record::new(
                                &context.configuration().unwrap().map_type.map_type_name
                            ),
                            mir::ir::RecordField::new(
                                concrete_map_type,
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
        fn compile_union_branch_including_map() {
            let context = Context::dummy(Default::default(), Default::default());
            let map_type = types::Map::new(
                types::None::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );
            let concrete_map_type =
                type_::compile_concrete_map(&map_type, context.types()).unwrap();

            assert_eq!(
                compile(
                    &context,
                    &IfType::new(
                        "y",
                        Variable::new("x", Position::fake()),
                        vec![IfTypeBranch::new(
                            types::Union::new(
                                map_type,
                                types::None::new(Position::fake()),
                                Position::fake()
                            ),
                            Variable::new("y", Position::fake()),
                        )],
                        None,
                        Position::fake(),
                    )
                    .into(),
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::Variable::new("x"),
                    vec![mir::ir::Alternative::new(
                        vec![concrete_map_type.into(), mir::types::Type::None],
                        "y",
                        mir::ir::Variable::new("y")
                    )],
                    None
                )
                .into())
            );
        }
    }

    mod records {
        use super::*;
        use hir::test::RecordFake;
        use pretty_assertions::assert_eq;

        #[test]
        fn compile_record_construction() {
            assert_eq!(
                compile(
                    &Context::dummy(
                        Default::default(),
                        [(
                            "r".into(),
                            vec![types::RecordField::new(
                                "x",
                                types::None::new(Position::fake())
                            )]
                        )]
                        .into_iter()
                        .collect()
                    ),
                    &RecordConstruction::new(
                        types::Record::fake("r"),
                        vec![RecordField::new(
                            "x",
                            None::new(Position::fake()),
                            Position::fake()
                        )],
                        Position::fake()
                    )
                    .into(),
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
                    &Context::dummy(
                        Default::default(),
                        [(
                            "r".into(),
                            vec![
                                types::RecordField::new("x", types::Number::new(Position::fake())),
                                types::RecordField::new("y", types::None::new(Position::fake()))
                            ]
                        )]
                        .into_iter()
                        .collect()
                    ),
                    &RecordConstruction::new(
                        types::Record::fake("r"),
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
                    &Context::dummy(
                        Default::default(),
                        [("r".into(), vec![])].into_iter().collect()
                    ),
                    &RecordConstruction::new(types::Record::fake("r"), vec![], Position::fake())
                        .into(),
                ),
                Ok(mir::ir::Record::new(mir::types::Record::new("r"), vec![]).into())
            );
        }

        #[test]
        fn compile_record_construction_with_reference_type() {
            assert_eq!(
                compile(
                    &Context::dummy(
                        [("r".into(), types::Record::fake("r").into())]
                            .into_iter()
                            .collect(),
                        [("r".into(), vec![])].into_iter().collect()
                    ),
                    &RecordConstruction::new(
                        types::Reference::new("r", Position::fake()),
                        vec![],
                        Position::fake()
                    )
                    .into(),
                ),
                Ok(mir::ir::Record::new(mir::types::Record::new("r"), vec![]).into())
            );
        }
    }

    mod try_operation {
        use crate::error_type;

        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn compile_with_none() {
            let error_type = error_type::compile_type();

            assert_eq!(
                compile(
                    &Context::dummy(Default::default(), Default::default()),
                    &TryOperation::new(
                        Some(types::None::new(Position::fake()).into()),
                        Variable::new("x", Position::fake()),
                        Position::fake(),
                    )
                    .into(),
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::TryOperation::new(
                        mir::ir::Variable::new("x"),
                        "$error",
                        error_type.clone(),
                        mir::ir::Variant::new(error_type, mir::ir::Variable::new("$error"))
                    ),
                    vec![mir::ir::Alternative::new(
                        vec![mir::types::Type::None],
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
            let error_type = error_type::compile_type();

            assert_eq!(
                compile(
                    &Context::dummy(Default::default(), Default::default()),
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
                ),
                Ok(mir::ir::Case::new(
                    mir::ir::TryOperation::new(
                        mir::ir::Variable::new("x"),
                        "$error",
                        error_type.clone(),
                        mir::ir::Variant::new(error_type, mir::ir::Variable::new("$error"))
                    ),
                    vec![mir::ir::Alternative::new(
                        vec![mir::types::Type::None, mir::types::Type::Number],
                        "$success",
                        mir::ir::Variable::new("$success"),
                    )],
                    None
                )
                .into())
            );
        }
    }
}
