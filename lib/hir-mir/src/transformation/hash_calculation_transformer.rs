use super::record_type_information_compiler;
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

pub fn transform(
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
                transform_any_function(context, list_type.element(), position)?.into(),
                value.clone(),
            ],
            position.clone(),
        )
        .into(),
        Type::Map(_) => Call::new(
            Some(
                types::Function::new(
                    vec![types::Reference::new(
                        &configuration.map_type.map_type_name,
                        position.clone(),
                    )
                    .into()],
                    types::Number::new(position.clone()),
                    position.clone(),
                )
                .into(),
            ),
            Variable::new(
                &configuration.map_type.hash.map_hash_function_name,
                position.clone(),
            ),
            vec![value.clone()],
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
            const VALUE_NAME: &str = "$x";
            let member_types = union_type_member_calculator::calculate(type_, context.types())?;

            IfType::new(
                VALUE_NAME,
                value.clone(),
                member_types
                    .iter()
                    .map(|type_| {
                        Ok(IfTypeBranch::new(
                            type_.clone(),
                            transform(
                                context,
                                &Variable::new(VALUE_NAME, position.clone()).into(),
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
        Type::Reference(reference) => transform(
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

pub fn transform_any_function(
    context: &CompileContext,
    type_: &Type,
    position: &Position,
) -> Result<Lambda, CompileError> {
    const ARGUMENT_NAME: &str = "$x";

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
                transform(
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

fn compile_any_hash_function_type(position: &Position) -> types::Function {
    types::Function::new(
        vec![types::Any::new(position.clone()).into()],
        types::Number::new(position.clone()),
        position.clone(),
    )
}
