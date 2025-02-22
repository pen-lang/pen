mod utilities;

use super::{
    super::{error::CompileError, type_},
    REFERENCE_COUNT_FUNCTION_DEFINITION_OPTIONS, expression, pointer,
};
use crate::{context::Context, record};

const ARGUMENT_NAME: &str = "_record";

pub fn compile_clone_function(
    context: &Context,
    definition: &mir::ir::TypeDefinition,
) -> Result<(), CompileError> {
    let record_type = mir::types::Record::new(definition.name());
    let fmm_record_type = type_::compile_record(context, &record_type);

    context.module_builder().define_function(
        utilities::get_clone_function_name(definition.name()),
        vec![fmm::ir::Argument::new(
            ARGUMENT_NAME,
            fmm_record_type.clone(),
        )],
        fmm_record_type.clone(),
        |builder| -> Result<_, CompileError> {
            let record = fmm::build::variable(ARGUMENT_NAME, fmm_record_type.clone());

            Ok(
                builder.return_(if type_::is_record_boxed(context, &record_type) {
                    clone_boxed(&builder, &record)?
                } else {
                    clone_unboxed(context, &builder, &record, &record_type)?
                }),
            )
        },
        REFERENCE_COUNT_FUNCTION_DEFINITION_OPTIONS.clone(),
    )?;

    Ok(())
}

pub fn compile_clone_unboxed_function(
    context: &Context,
    definition: &mir::ir::TypeDefinition,
) -> Result<(), CompileError> {
    let record_type = mir::types::Record::new(definition.name());
    let fmm_record_type = type_::compile_unboxed_record(context, &record_type);

    context.module_builder().define_function(
        utilities::get_clone_unboxed_function_name(definition.name()),
        vec![fmm::ir::Argument::new(
            ARGUMENT_NAME,
            fmm_record_type.clone(),
        )],
        fmm_record_type.clone(),
        |builder| -> Result<_, CompileError> {
            let record = fmm::build::variable(ARGUMENT_NAME, fmm_record_type.clone());
            Ok(builder.return_(fmm::build::record(
                context.types()[record_type.name()]
                    .fields()
                    .iter()
                    .enumerate()
                    .map(|(index, type_)| {
                        expression::clone(
                            context,
                            &builder,
                            &record::get_unboxed_field(&builder, &record, index)?,
                            type_,
                        )
                    })
                    .collect::<Result<_, _>>()?,
            )))
        },
        REFERENCE_COUNT_FUNCTION_DEFINITION_OPTIONS.clone(),
    )?;

    Ok(())
}

fn clone_boxed(
    builder: &fmm::build::InstructionBuilder,
    record: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    pointer::clone(builder, record)
}

pub fn clone_unboxed(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    record: &fmm::build::TypedExpression,
    record_type: &mir::types::Record,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(builder.call(
        fmm::build::variable(
            utilities::get_clone_unboxed_function_name(record_type.name()),
            utilities::compile_clone_unboxed_function_type(context, record_type),
        ),
        vec![record.clone()],
    )?)
}

pub fn compile_drop_function(
    context: &Context,
    definition: &mir::ir::TypeDefinition,
) -> Result<(), CompileError> {
    let record_type = mir::types::Record::new(definition.name());
    let fmm_record_type = type_::compile_record(context, &record_type);

    context.module_builder().define_function(
        utilities::get_drop_function_name(definition.name()),
        vec![fmm::ir::Argument::new(
            ARGUMENT_NAME,
            fmm_record_type.clone(),
        )],
        fmm::types::void_type(),
        |builder| -> Result<_, CompileError> {
            let record = fmm::build::variable(ARGUMENT_NAME, fmm_record_type.clone());

            if type_::is_record_boxed(context, &record_type) {
                drop_boxed(context, &builder, &record, &record_type)?
            } else {
                drop_unboxed(context, &builder, &record, &record_type)?;
            }

            Ok(builder.return_(fmm::ir::void_value()))
        },
        REFERENCE_COUNT_FUNCTION_DEFINITION_OPTIONS.clone(),
    )?;

    Ok(())
}

fn drop_boxed(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    record: &fmm::build::TypedExpression,
    record_type: &mir::types::Record,
) -> Result<(), CompileError> {
    pointer::drop(builder, record, |builder| {
        drop_unboxed(
            context,
            builder,
            &record::load(context, builder, record, record_type)?,
            record_type,
        )
    })
}

fn drop_unboxed(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    record: &fmm::build::TypedExpression,
    record_type: &mir::types::Record,
) -> Result<(), CompileError> {
    for (index, type_) in context.types()[record_type.name()]
        .fields()
        .iter()
        .enumerate()
    {
        expression::drop(
            context,
            builder,
            &record::get_unboxed_field(builder, record, index)?,
            type_,
        )?;
    }

    Ok(())
}

pub fn compile_synchronize_function(
    context: &Context,
    definition: &mir::ir::TypeDefinition,
) -> Result<(), CompileError> {
    let record_type = mir::types::Record::new(definition.name());
    let fmm_record_type = type_::compile_record(context, &record_type);

    context.module_builder().define_function(
        utilities::get_synchronize_function_name(definition.name()),
        vec![fmm::ir::Argument::new(
            ARGUMENT_NAME,
            fmm_record_type.clone(),
        )],
        fmm::types::void_type(),
        |builder| -> Result<_, CompileError> {
            let record = fmm::build::variable(ARGUMENT_NAME, fmm_record_type.clone());

            if type_::is_record_boxed(context, &record_type) {
                synchronize_boxed(context, &builder, &record, &record_type)?
            } else {
                synchronize_unboxed(context, &builder, &record, &record_type)?;
            }

            Ok(builder.return_(fmm::ir::void_value()))
        },
        REFERENCE_COUNT_FUNCTION_DEFINITION_OPTIONS.clone(),
    )?;

    Ok(())
}

fn synchronize_boxed(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    record: &fmm::build::TypedExpression,
    record_type: &mir::types::Record,
) -> Result<(), CompileError> {
    pointer::synchronize(builder, record, |builder| {
        synchronize_unboxed(
            context,
            builder,
            &record::load(context, builder, record, record_type)?,
            record_type,
        )
    })
}

fn synchronize_unboxed(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    record: &fmm::build::TypedExpression,
    record_type: &mir::types::Record,
) -> Result<(), CompileError> {
    for (index, type_) in context.types()[record_type.name()]
        .fields()
        .iter()
        .enumerate()
    {
        expression::synchronize(
            context,
            builder,
            &record::get_unboxed_field(builder, record, index)?,
            type_,
        )?;
    }

    Ok(())
}

pub fn clone(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
    record_type: &mir::types::Record,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(builder.call(
        fmm::build::variable(
            utilities::get_clone_function_name(record_type.name()),
            utilities::compile_clone_function_type(context, record_type),
        ),
        vec![expression.clone()],
    )?)
}

pub fn drop(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
    record_type: &mir::types::Record,
) -> Result<(), CompileError> {
    builder.call(
        fmm::build::variable(
            utilities::get_drop_function_name(record_type.name()),
            utilities::compile_drop_function_type(context, record_type),
        ),
        vec![expression.clone()],
    )?;

    Ok(())
}

pub fn synchronize(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
    record_type: &mir::types::Record,
) -> Result<(), CompileError> {
    builder.call(
        fmm::build::variable(
            utilities::get_synchronize_function_name(record_type.name()),
            utilities::compile_synchronize_function_type(context, record_type),
        ),
        vec![expression.clone()],
    )?;

    Ok(())
}
