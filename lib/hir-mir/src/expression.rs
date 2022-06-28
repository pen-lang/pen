use super::{
    context::CompileContext,
    transformation::{boolean_operation, equal_operation, if_list, not_equal_operation},
    type_, CompileError,
};
use crate::{
    concurrency_configuration::MODULE_LOCAL_SPAWN_FUNCTION_NAME,
    downcast,
    transformation::{if_map, list_literal, map_literal},
};
use fnv::FnvHashMap;
use hir::{
    analysis::{
        record_field_resolver, type_canonicalizer, type_equality_checker,
        union_type_member_calculator, AnalysisError,
    },
    ir::*,
    types::{self, Type},
};

pub fn compile(
    context: &CompileContext,
    expression: &Expression,
) -> Result<mir::ir::Expression, CompileError> {
    let compile = |expression| compile(context, expression);

    Ok(match expression {
        Expression::Boolean(boolean) => mir::ir::Expression::Boolean(boolean.value()),
        Expression::Call(call) => mir::ir::Call::new(
            type_::compile(
                context,
                call.function_type()
                    .ok_or_else(|| AnalysisError::TypeNotInferred(call.position().clone()))?,
            )?
            .into_function()
            .ok_or_else(|| AnalysisError::FunctionExpected(call.position().clone()))?,
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
        Expression::IfList(if_) => compile(&if_list::transform(context, if_)?)?,
        Expression::IfMap(if_) => compile(&if_map::transform(context, if_)?)?,
        Expression::IfType(if_) => mir::ir::Case::new(
            compile(if_.argument())?,
            if_.branches()
                .iter()
                .map(|branch| {
                    compile_alternatives(context, if_.name(), branch.type_(), branch.expression())
                })
                .collect::<Result<Vec<_>, CompileError>>()?
                .into_iter()
                .flatten()
                .chain(if let Some(branch) = if_.else_() {
                    if !type_equality_checker::check(
                        branch.type_().unwrap(),
                        &types::Any::new(if_.position().clone()).into(),
                        context.types(),
                    )? {
                        compile_alternatives(
                            context,
                            if_.name(),
                            branch.type_().unwrap(),
                            branch.expression(),
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
        Expression::List(list) => compile(&list_literal::transform(
            list,
            &context.configuration()?.list_type,
        ))?,
        Expression::ListComprehension(comprehension) => {
            compile_list_comprehension(context, comprehension)?
        }
        Expression::Map(map) => compile(&map_literal::transform(context, map)?)?,
        Expression::MapIterationComprehension(comprehension) => {
            compile_map_iteration_comprehension(context, comprehension)?
        }
        Expression::None(_) => mir::ir::Expression::None,
        Expression::Number(number) => mir::ir::Expression::Number(number.value()),
        Expression::Operation(operation) => compile_operation(context, operation)?,
        Expression::RecordConstruction(construction) => {
            let field_types = record_field_resolver::resolve(
                construction.type_(),
                construction.position(),
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
                    deconstruction.position(),
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
                            update.position(),
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
                    compile(thunk.expression())?,
                    type_::compile(
                        context,
                        thunk.type_().ok_or_else(|| {
                            AnalysisError::TypeNotInferred(thunk.position().clone())
                        })?,
                    )?,
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
                | Type::None(_)
                | Type::Number(_)
                | Type::Record(_)
                | Type::String(_) => {
                    mir::ir::Variant::new(type_::compile(context, &from)?, argument).into()
                }
                Type::Function(function_type) => {
                    let concrete_type =
                        type_::compile_concrete_function(function_type, context.types())?;

                    mir::ir::Variant::new(
                        concrete_type.clone(),
                        mir::ir::Record::new(concrete_type, vec![argument]),
                    )
                    .into()
                }
                Type::List(list_type) => {
                    if to.is_list() {
                        argument
                    } else {
                        let concrete_type =
                            type_::compile_concrete_list(list_type, context.types())?;

                        mir::ir::Variant::new(
                            concrete_type.clone(),
                            mir::ir::Record::new(concrete_type, vec![argument]),
                        )
                        .into()
                    }
                }
                Type::Map(map_type) => {
                    if to.is_map() {
                        argument
                    } else {
                        let concrete_type = type_::compile_concrete_map(map_type, context.types())?;

                        mir::ir::Variant::new(
                            concrete_type.clone(),
                            mir::ir::Record::new(concrete_type, vec![argument]),
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
    context: &CompileContext,
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
            compile(context, lambda.body())?,
            type_::compile(context, lambda.result_type())?,
        ),
        mir::ir::Variable::new(CLOSURE_NAME),
    )
    .into())
}

fn compile_alternatives(
    context: &CompileContext,
    name: &str,
    type_: &Type,
    expression: &Expression,
) -> Result<Vec<mir::ir::Alternative>, CompileError> {
    let type_ = type_canonicalizer::canonicalize(type_, context.types())?;
    let expression = compile(context, expression)?;

    union_type_member_calculator::calculate(&type_, context.types())?
        .into_iter()
        .map(|member_type| {
            let compiled_member_type = type_::compile(context, &member_type)?;

            Ok(match &member_type {
                Type::Function(function_type) => compile_generic_type_alternative(
                    name,
                    &expression,
                    &type_,
                    &compiled_member_type,
                    &type_::compile_concrete_function(function_type, context.types())?,
                ),
                Type::List(list_type) => compile_generic_type_alternative(
                    name,
                    &expression,
                    &type_,
                    &compiled_member_type,
                    &type_::compile_concrete_list(list_type, context.types())?,
                ),
                Type::Map(map_type) => compile_generic_type_alternative(
                    name,
                    &expression,
                    &type_,
                    &compiled_member_type,
                    &type_::compile_concrete_map(map_type, context.types())?,
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

fn compile_list_comprehension(
    context: &CompileContext,
    comprehension: &ListComprehension,
) -> Result<mir::ir::Expression, CompileError> {
    let compile = |expression| compile(context, expression);

    const CLOSURE_NAME: &str = "$loop";
    const LIST_NAME: &str = "$list";

    let position = comprehension.position();
    let input_element_type = comprehension
        .input_type()
        .ok_or_else(|| AnalysisError::TypeNotInferred(position.clone()))?;
    let output_element_type = comprehension.output_type();
    let list_type = type_::compile_list(context)?;

    Ok(mir::ir::Call::new(
        mir::types::Function::new(
            vec![mir::types::Function::new(vec![], list_type.clone()).into()],
            list_type.clone(),
        ),
        mir::ir::Variable::new(&context.configuration()?.list_type.lazy_function_name),
        vec![mir::ir::LetRecursive::new(
            mir::ir::FunctionDefinition::new(
                CLOSURE_NAME,
                vec![],
                mir::ir::LetRecursive::new(
                    mir::ir::FunctionDefinition::new(
                        CLOSURE_NAME,
                        vec![mir::ir::Argument::new(LIST_NAME, list_type.clone())],
                        compile(
                            &IfList::new(
                                Some(input_element_type.clone()),
                                Variable::new(LIST_NAME, position.clone()),
                                comprehension.element_name(),
                                LIST_NAME,
                                List::new(
                                    output_element_type.clone(),
                                    vec![
                                        ListElement::Single(comprehension.element().clone()),
                                        ListElement::Multiple(
                                            Call::new(
                                                Some(
                                                    types::Function::new(
                                                        vec![types::List::new(
                                                            input_element_type.clone(),
                                                            position.clone(),
                                                        )
                                                        .into()],
                                                        types::List::new(
                                                            output_element_type.clone(),
                                                            position.clone(),
                                                        ),
                                                        position.clone(),
                                                    )
                                                    .into(),
                                                ),
                                                Variable::new(CLOSURE_NAME, position.clone()),
                                                vec![Variable::new(LIST_NAME, position.clone())
                                                    .into()],
                                                position.clone(),
                                            )
                                            .into(),
                                        ),
                                    ],
                                    position.clone(),
                                ),
                                List::new(output_element_type.clone(), vec![], position.clone()),
                                position.clone(),
                            )
                            .into(),
                        )?,
                        list_type.clone(),
                    ),
                    mir::ir::Call::new(
                        mir::types::Function::new(
                            vec![list_type.clone().into()],
                            list_type.clone(),
                        ),
                        mir::ir::Variable::new(CLOSURE_NAME),
                        vec![compile(comprehension.list())?],
                    ),
                ),
                list_type,
            ),
            mir::ir::Variable::new(CLOSURE_NAME),
        )
        .into()],
    )
    .into())
}

fn compile_map_iteration_comprehension(
    context: &CompileContext,
    comprehension: &MapIterationComprehension,
) -> Result<mir::ir::Expression, CompileError> {
    const CLOSURE_NAME: &str = "$loop";

    let list_type = type_::compile_list(context)?;
    let map_type = type_::compile_map(context)?;
    let definition = compile_map_iteration_function_definition(context, comprehension)?;

    Ok(mir::ir::Call::new(
        mir::types::Function::new(
            vec![mir::types::Function::new(vec![], list_type.clone()).into()],
            list_type.clone(),
        ),
        mir::ir::Variable::new(&context.configuration()?.list_type.lazy_function_name),
        vec![mir::ir::LetRecursive::new(
            mir::ir::FunctionDefinition::new(
                CLOSURE_NAME,
                vec![],
                mir::ir::LetRecursive::new(
                    definition.clone(),
                    mir::ir::Call::new(
                        mir::types::Function::new(
                            vec![mir::types::Type::Variant],
                            list_type.clone(),
                        ),
                        mir::ir::Variable::new(definition.name()),
                        vec![mir::ir::Call::new(
                            mir::types::Function::new(
                                vec![map_type.into()],
                                mir::types::Type::Variant,
                            ),
                            mir::ir::Variable::new(
                                &context
                                    .configuration()?
                                    .map_type
                                    .iteration
                                    .iterate_function_name,
                            ),
                            vec![compile(context, comprehension.map())?],
                        )
                        .into()],
                    ),
                ),
                list_type,
            ),
            mir::ir::Variable::new(CLOSURE_NAME),
        )
        .into()],
    )
    .into())
}

fn compile_map_iteration_function_definition(
    context: &CompileContext,
    comprehension: &MapIterationComprehension,
) -> Result<mir::ir::FunctionDefinition, CompileError> {
    const CLOSURE_NAME: &str = "$loop";
    const ITERATOR_NAME: &str = "$iterator";

    let iteration_configuration = &context.configuration()?.map_type.iteration;
    let position = comprehension.position();
    let any_type = Type::from(types::Any::new(position.clone()));
    let key_type = comprehension
        .key_type()
        .ok_or_else(|| AnalysisError::TypeNotInferred(position.clone()))?;
    let value_type = comprehension
        .value_type()
        .ok_or_else(|| AnalysisError::TypeNotInferred(position.clone()))?;
    let element_type = comprehension.element_type();
    let iterator_type = Type::from(types::Reference::new(
        &iteration_configuration.iterator_type_name,
        position.clone(),
    ));
    let iterator_or_none_type = types::Union::new(
        iterator_type.clone(),
        types::None::new(position.clone()),
        position.clone(),
    );
    let iterator_variable = Variable::new(ITERATOR_NAME, position.clone());
    let compile_key_value_function_call = |name, type_| {
        downcast::compile(
            context,
            &any_type,
            type_,
            &Call::new(
                Some(
                    types::Function::new(
                        vec![iterator_type.clone()],
                        any_type.clone(),
                        position.clone(),
                    )
                    .into(),
                ),
                Variable::new(name, position.clone()),
                vec![iterator_variable.clone().into()],
                position.clone(),
            )
            .into(),
        )
    };

    Ok(mir::ir::FunctionDefinition::new(
        CLOSURE_NAME,
        vec![mir::ir::Argument::new(
            ITERATOR_NAME,
            mir::types::Type::Variant,
        )],
        compile(
            context,
            &IfType::new(
                ITERATOR_NAME,
                iterator_variable.clone(),
                vec![IfTypeBranch::new(
                    iterator_type.clone(),
                    Let::new(
                        Some(comprehension.key_name().into()),
                        Some(key_type.clone()),
                        compile_key_value_function_call(
                            &iteration_configuration.key_function_name,
                            key_type,
                        )?,
                        Let::new(
                            Some(comprehension.value_name().into()),
                            Some(value_type.clone()),
                            compile_key_value_function_call(
                                &iteration_configuration.value_function_name,
                                value_type,
                            )?,
                            List::new(
                                element_type.clone(),
                                vec![
                                    ListElement::Single(comprehension.element().clone()),
                                    ListElement::Multiple(
                                        Call::new(
                                            Some(
                                                types::Function::new(
                                                    vec![iterator_or_none_type.clone().into()],
                                                    types::List::new(
                                                        element_type.clone(),
                                                        position.clone(),
                                                    ),
                                                    position.clone(),
                                                )
                                                .into(),
                                            ),
                                            Variable::new(CLOSURE_NAME, position.clone()),
                                            vec![Call::new(
                                                Some(
                                                    types::Function::new(
                                                        vec![iterator_type.clone()],
                                                        iterator_or_none_type,
                                                        position.clone(),
                                                    )
                                                    .into(),
                                                ),
                                                Variable::new(
                                                    &iteration_configuration.rest_function_name,
                                                    position.clone(),
                                                ),
                                                vec![iterator_variable.into()],
                                                position.clone(),
                                            )
                                            .into()],
                                            position.clone(),
                                        )
                                        .into(),
                                    ),
                                ],
                                position.clone(),
                            ),
                            position.clone(),
                        ),
                        position.clone(),
                    ),
                )],
                Some(ElseBranch::new(
                    Some(types::None::new(position.clone()).into()),
                    List::new(element_type.clone(), vec![], position.clone()),
                    position.clone(),
                )),
                position.clone(),
            )
            .into(),
        )?,
        type_::compile_list(context)?,
    ))
}

fn compile_operation(
    context: &CompileContext,
    operation: &Operation,
) -> Result<mir::ir::Expression, CompileError> {
    let compile = |expression| compile(context, expression);

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
        Operation::Spawn(operation) => compile_spawn_operation(context, operation)?,
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
                    _ => compile(&equal_operation::transform(context, operation)?)?,
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
            let success_type = operation
                .type_()
                .ok_or_else(|| AnalysisError::TypeNotInferred(operation.position().clone()))?;
            let error_type = type_::compile(
                context,
                &types::Reference::new(
                    &context.configuration()?.error_type.error_type_name,
                    operation.position().clone(),
                )
                .into(),
            )?;

            mir::ir::Case::new(
                mir::ir::TryOperation::new(
                    compile(operation.expression())?,
                    "$error",
                    error_type.clone(),
                    mir::ir::Variant::new(error_type, mir::ir::Variable::new("$error")),
                ),
                compile_alternatives(
                    context,
                    "$success",
                    success_type,
                    &Variable::new("$success", operation.position().clone()).into(),
                )?,
                None,
            )
            .into()
        }
    })
}

fn compile_spawn_operation(
    context: &CompileContext,
    operation: &SpawnOperation,
) -> Result<mir::ir::Expression, CompileError> {
    const ANY_THUNK_NAME: &str = "$any_thunk";
    const THUNK_NAME: &str = "$thunk";

    let position = operation.position();
    let body = operation.function().body();
    let result_type = operation.function().result_type();
    let any_type = Type::from(types::Any::new(position.clone()));
    let thunk_type = types::Function::new(vec![], any_type.clone(), position.clone()).into();
    let mir_thunk_type = type_::compile(context, &thunk_type)?;

    Ok(mir::ir::Let::new(
        ANY_THUNK_NAME,
        mir_thunk_type.clone(),
        mir::ir::Call::new(
            type_::compile_spawn_function(),
            mir::ir::Variable::new(MODULE_LOCAL_SPAWN_FUNCTION_NAME),
            vec![mir::ir::LetRecursive::new(
                mir::ir::FunctionDefinition::thunk(
                    ANY_THUNK_NAME,
                    compile(
                        context,
                        &TypeCoercion::new(
                            result_type.clone(),
                            any_type.clone(),
                            body.clone(),
                            body.position().clone(),
                        )
                        .into(),
                    )?,
                    type_::compile(context, &any_type)?,
                ),
                mir::ir::Synchronize::new(mir_thunk_type, mir::ir::Variable::new(ANY_THUNK_NAME)),
            )
            .into()],
        ),
        mir::ir::LetRecursive::new(
            mir::ir::FunctionDefinition::new(
                THUNK_NAME,
                vec![],
                compile(
                    context,
                    &downcast::compile(
                        context,
                        &any_type,
                        result_type,
                        &Call::new(
                            Some(thunk_type),
                            Variable::new(ANY_THUNK_NAME, position.clone()),
                            vec![],
                            position.clone(),
                        )
                        .into(),
                    )?,
                )?,
                type_::compile(context, result_type)?,
            ),
            mir::ir::Variable::new(THUNK_NAME),
        ),
    )
    .into())
}

fn compile_record_fields(
    context: &CompileContext,
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
    use position::{test::PositionFake, Position};

    fn compile_expression(expression: &Expression) -> Result<mir::ir::Expression, CompileError> {
        compile(
            &CompileContext::dummy(Default::default(), Default::default()),
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
            let context = CompileContext::dummy(Default::default(), Default::default());
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
                        concrete_function_type.clone(),
                        "y",
                        mir::ir::Let::new(
                            "y",
                            type_::compile_function(&function_type, &context).unwrap(),
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
            let context = CompileContext::dummy(Default::default(), Default::default());
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
                        concrete_list_type.clone(),
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
            let context = CompileContext::dummy(Default::default(), Default::default());
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
                        ),
                    ],
                    None
                )
                .into())
            );
        }

        #[test]
        fn compile_map_branch() {
            let context = CompileContext::dummy(Default::default(), Default::default());
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
                        concrete_map_type.clone(),
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
            let context = CompileContext::dummy(Default::default(), Default::default());
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
                    vec![
                        mir::ir::Alternative::new(
                            concrete_map_type.clone(),
                            "y",
                            mir::ir::Let::new(
                                "y",
                                mir::types::Type::Variant,
                                mir::ir::Variant::new(
                                    concrete_map_type,
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
                        ),
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
                    &CompileContext::dummy(
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
                        types::Record::new("r", Position::fake()),
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
                    &CompileContext::dummy(
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
                    &CompileContext::dummy(
                        Default::default(),
                        [("r".into(), vec![])].into_iter().collect()
                    ),
                    &RecordConstruction::new(
                        types::Record::new("r", Position::fake()),
                        vec![],
                        Position::fake()
                    )
                    .into(),
                ),
                Ok(mir::ir::Record::new(mir::types::Record::new("r"), vec![]).into())
            );
        }

        #[test]
        fn compile_record_construction_with_reference_type() {
            assert_eq!(
                compile(
                    &CompileContext::dummy(
                        [("r".into(), types::Record::new("r", Position::fake()).into())]
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
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn compile_with_none() {
            let error_type = mir::types::Record::new("error");

            assert_eq!(
                compile(
                    &CompileContext::dummy(
                        [(
                            "error".into(),
                            types::Record::new("error", Position::fake()).into()
                        )]
                        .into_iter()
                        .collect(),
                        Default::default()
                    ),
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
                    &CompileContext::dummy(
                        [(
                            "error".into(),
                            types::Record::new("error", Position::fake()).into()
                        )]
                        .into_iter()
                        .collect(),
                        Default::default()
                    ),
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
                        ),
                    ],
                    None
                )
                .into())
            );
        }
    }

    mod spawn_operation {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn compile_spawn_operation() {
            let thunk_type = mir::types::Function::new(vec![], mir::types::Type::Variant);

            assert_eq!(
                compile_expression(
                    &SpawnOperation::new(
                        Lambda::new(
                            vec![],
                            types::Number::new(Position::fake()),
                            Number::new(42.0, Position::fake()),
                            Position::fake()
                        ),
                        Position::fake(),
                    )
                    .into(),
                ),
                Ok(mir::ir::Let::new(
                    "$any_thunk",
                    thunk_type.clone(),
                    mir::ir::Call::new(
                        type_::compile_spawn_function(),
                        mir::ir::Variable::new(MODULE_LOCAL_SPAWN_FUNCTION_NAME),
                        vec![mir::ir::LetRecursive::new(
                            mir::ir::FunctionDefinition::thunk(
                                "$any_thunk",
                                mir::ir::Variant::new(
                                    mir::types::Type::Number,
                                    mir::ir::Expression::Number(42.0)
                                ),
                                mir::types::Type::Variant
                            ),
                            mir::ir::Synchronize::new(
                                thunk_type.clone(),
                                mir::ir::Variable::new("$any_thunk")
                            ),
                        )
                        .into()]
                    ),
                    mir::ir::LetRecursive::new(
                        mir::ir::FunctionDefinition::new(
                            "$thunk",
                            vec![],
                            mir::ir::Case::new(
                                mir::ir::Call::new(
                                    thunk_type,
                                    mir::ir::Variable::new("$any_thunk"),
                                    vec![]
                                ),
                                vec![mir::ir::Alternative::new(
                                    mir::types::Type::Number,
                                    "$value",
                                    mir::ir::Variable::new("$value")
                                )],
                                None,
                            ),
                            mir::types::Type::Number
                        ),
                        mir::ir::Variable::new("$thunk"),
                    ),
                )
                .into())
            );
        }

        #[test]
        fn compile_spawn_operation_with_any_type() {
            let thunk_type = mir::types::Function::new(vec![], mir::types::Type::Variant);

            assert_eq!(
                compile_expression(
                    &SpawnOperation::new(
                        Lambda::new(
                            vec![],
                            types::Any::new(Position::fake()),
                            Variable::new("x", Position::fake()),
                            Position::fake()
                        ),
                        Position::fake(),
                    )
                    .into(),
                ),
                Ok(mir::ir::Let::new(
                    "$any_thunk",
                    thunk_type.clone(),
                    mir::ir::Call::new(
                        type_::compile_spawn_function(),
                        mir::ir::Variable::new(MODULE_LOCAL_SPAWN_FUNCTION_NAME),
                        vec![mir::ir::LetRecursive::new(
                            mir::ir::FunctionDefinition::thunk(
                                "$any_thunk",
                                mir::ir::Variable::new("x"),
                                mir::types::Type::Variant
                            ),
                            mir::ir::Synchronize::new(
                                thunk_type.clone(),
                                mir::ir::Variable::new("$any_thunk")
                            ),
                        )
                        .into()]
                    ),
                    mir::ir::LetRecursive::new(
                        mir::ir::FunctionDefinition::new(
                            "$thunk",
                            vec![],
                            mir::ir::Call::new(
                                thunk_type,
                                mir::ir::Variable::new("$any_thunk"),
                                vec![]
                            ),
                            mir::types::Type::Variant
                        ),
                        mir::ir::Variable::new("$thunk"),
                    ),
                )
                .into())
            );
        }
    }
}
