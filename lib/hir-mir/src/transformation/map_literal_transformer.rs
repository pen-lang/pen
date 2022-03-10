use super::{equal_operation_transformer, record_type_information_compiler};
use crate::{context::CompileContext, CompileError};
use hir::{
    analysis::{type_comparability_checker, type_resolver, union_type_member_calculator},
    ir::*,
    types::{self, Type},
};
use position::Position;

const NONE_HASH: f64 = 0.0;
const BOOLEAN_TRUE_HASH: f64 = 1.0;
const BOOLEAN_FALSE_HASH: f64 = 2.0;
const ARGUMENT_NAME: &str = "$x";

pub fn transform(context: &CompileContext, map: &Map) -> Result<Expression, CompileError> {
    transform_map(
        context,
        map.key_type(),
        map.value_type(),
        map.elements(),
        map.position(),
    )
}

fn transform_map(
    context: &CompileContext,
    key_type: &Type,
    value_type: &Type,
    elements: &[MapElement],
    position: &Position,
) -> Result<Expression, CompileError> {
    let configuration = &context.configuration()?.map_type;
    let any_map_type = types::Reference::new(configuration.map_type_name.clone(), position.clone());

    Ok(match elements {
        [] => Call::new(
            Some(types::Function::new(vec![], any_map_type, position.clone()).into()),
            Variable::new(&configuration.empty_function_name, position.clone()),
            vec![
                equal_operation_transformer::compile_any_equal_function(
                    context, key_type, position,
                )?
                .into(),
                compile_any_hash_function(context, key_type, position)?.into(),
                equal_operation_transformer::compile_any_equal_function(
                    context, value_type, position,
                )?
                .into(),
                compile_any_hash_function(context, value_type, position)?.into(),
            ],
            position.clone(),
        )
        .into(),
        [element, ..] => {
            let rest_expression =
                transform_map(context, key_type, value_type, &elements[1..], position)?;

            match element {
                MapElement::Insertion(entry) => Call::new(
                    Some(
                        types::Function::new(
                            vec![
                                any_map_type.clone().into(),
                                types::Any::new(position.clone()).into(),
                                types::Any::new(position.clone()).into(),
                            ],
                            any_map_type,
                            position.clone(),
                        )
                        .into(),
                    ),
                    Variable::new(&configuration.set_function_name, position.clone()),
                    vec![
                        rest_expression,
                        TypeCoercion::new(
                            key_type.clone(),
                            types::Any::new(position.clone()),
                            entry.key().clone(),
                            position.clone(),
                        )
                        .into(),
                        TypeCoercion::new(
                            value_type.clone(),
                            types::Any::new(position.clone()),
                            entry.value().clone(),
                            position.clone(),
                        )
                        .into(),
                    ],
                    position.clone(),
                )
                .into(),
                MapElement::Map(expression) => Call::new(
                    Some(
                        types::Function::new(
                            vec![any_map_type.clone().into(), any_map_type.clone().into()],
                            any_map_type,
                            position.clone(),
                        )
                        .into(),
                    ),
                    Variable::new(&configuration.merge_function_name, position.clone()),
                    vec![expression.clone(), rest_expression],
                    position.clone(),
                )
                .into(),
                MapElement::Removal(key) => Call::new(
                    Some(
                        types::Function::new(
                            vec![
                                any_map_type.clone().into(),
                                types::Any::new(position.clone()).into(),
                            ],
                            any_map_type,
                            position.clone(),
                        )
                        .into(),
                    ),
                    Variable::new(&configuration.delete_function_name, position.clone()),
                    vec![
                        rest_expression,
                        TypeCoercion::new(
                            key_type.clone(),
                            types::Any::new(position.clone()),
                            key.clone(),
                            position.clone(),
                        )
                        .into(),
                    ],
                    position.clone(),
                )
                .into(),
            }
        }
    })
}

fn compile_any_hash_function(
    context: &CompileContext,
    type_: &Type,
    position: &Position,
) -> Result<Lambda, CompileError> {
    Ok(Lambda::new(
        vec![Argument::new(
            ARGUMENT_NAME,
            types::Any::new(position.clone()),
        )],
        types::Number::new(position.clone()),
        IfType::new(
            ARGUMENT_NAME,
            Variable::new(ARGUMENT_NAME, position.clone()),
            vec![IfTypeBranch::new(
                type_.clone(),
                transform_hash_calculation(
                    context,
                    &Variable::new(ARGUMENT_NAME, position.clone()).into(),
                    type_,
                    position,
                )?,
            )],
            None,
            position.clone(),
        ),
        position.clone(),
    ))
}

pub fn transform_hash_calculation(
    context: &CompileContext,
    value: &Expression,
    type_: &Type,
    position: &Position,
) -> Result<Expression, CompileError> {
    let configuration = context.configuration()?;

    Ok(match type_ {
        Type::Boolean(_) => If::new(
            value.clone(),
            Number::new(BOOLEAN_TRUE_HASH, position.clone()),
            Number::new(BOOLEAN_FALSE_HASH, position.clone()),
            position.clone(),
        )
        .into(),
        Type::List(list_type) => Call::new(
            Some(
                types::Function::new(
                    vec![
                        compile_any_hash_function_type(position).into(),
                        types::Reference::new(
                            &configuration.list_type.list_type_name,
                            position.clone(),
                        )
                        .into(),
                    ],
                    types::Number::new(position.clone()),
                    position.clone(),
                )
                .into(),
            ),
            Variable::new(
                &configuration.map_type.hash.list_hash_function_name,
                position.clone(),
            ),
            vec![
                compile_any_hash_function(context, list_type.element(), position)?.into(),
                value.clone(),
            ],
            position.clone(),
        )
        .into(),
        Type::Map(map_type) => Call::new(
            Some(
                types::Function::new(
                    vec![
                        compile_any_hash_function_type(position).into(),
                        types::Reference::new(
                            &configuration.map_type.map_type_name,
                            position.clone(),
                        )
                        .into(),
                    ],
                    types::Number::new(position.clone()),
                    position.clone(),
                )
                .into(),
            ),
            Variable::new(
                &configuration.map_type.hash.map_hash_function_name,
                position.clone(),
            ),
            vec![
                compile_any_hash_function(context, map_type.value(), position)?.into(),
                value.clone(),
            ],
            position.clone(),
        )
        .into(),
        Type::Number(_) => compile_concrete_hash_function_call(
            &configuration.map_type.hash.number_hash_function_name,
            value,
            type_,
            position,
        ),
        Type::Record(record_type) => {
            if !type_comparability_checker::check(type_, context.types(), context.records())? {
                return Err(CompileError::InvalidRecordEqualOperation(position.clone()));
            }

            compile_concrete_hash_function_call(
                record_type_information_compiler::compile_hash_function_name(record_type),
                value,
                type_,
                position,
            )
        }
        Type::String(_) => compile_concrete_hash_function_call(
            &configuration.map_type.hash.string_hash_function_name,
            value,
            type_,
            position,
        ),
        Type::Union(_) => {
            let member_types = union_type_member_calculator::calculate(type_, context.types())?;

            IfType::new(
                ARGUMENT_NAME,
                value.clone(),
                member_types
                    .iter()
                    .map(|type_| {
                        Ok(IfTypeBranch::new(
                            type_.clone(),
                            transform_hash_calculation(
                                context,
                                &Variable::new(ARGUMENT_NAME, position.clone()).into(),
                                type_,
                                position,
                            )?,
                        ))
                    })
                    .collect::<Result<_, CompileError>>()?,
                None,
                position.clone(),
            )
            .into()
        }
        Type::Reference(reference) => transform_hash_calculation(
            context,
            value,
            &type_resolver::resolve(reference, context.types())?,
            position,
        )?,
        Type::Any(_) | Type::Function(_) | Type::None(_) => {
            Number::new(NONE_HASH, position.clone()).into()
        }
    })
}

fn compile_concrete_hash_function_call(
    name: impl Into<String>,
    value: &Expression,
    type_: &Type,
    position: &Position,
) -> Expression {
    Call::new(
        Some(
            types::Function::new(
                vec![type_.clone()],
                types::Number::new(position.clone()),
                position.clone(),
            )
            .into(),
        ),
        Variable::new(name.into(), position.clone()),
        vec![value.clone()],
        position.clone(),
    )
    .into()
}

fn compile_any_hash_function_type(position: &Position) -> types::Function {
    types::Function::new(
        vec![types::Any::new(position.clone()).into()],
        types::Number::new(position.clone()),
        position.clone(),
    )
}
