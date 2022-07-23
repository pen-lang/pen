pub(super) mod utilities;

use super::{
    super::{error::CompileError, type_},
    expression, pointer,
};
use crate::{context::Context, record};

const ARGUMENT_NAME: &str = "_record";

pub fn compile_clone_function(
    context: &Context,
    definition: &mir::ir::TypeDefinition,
) -> Result<(), CompileError> {
    let record_type = mir::types::Record::new(definition.name());
    let fmm_record_type = type_::compile_record(&record_type, context.types());

    context.module_builder().define_function(
        utilities::get_clone_function_name(definition.name()),
        vec![fmm::ir::Argument::new(
            ARGUMENT_NAME,
            fmm_record_type.clone(),
        )],
        |builder| -> Result<_, CompileError> {
            let record = fmm::build::variable(ARGUMENT_NAME, fmm_record_type.clone());

            Ok(
                builder.return_(if type_::is_record_boxed(&record_type, context.types()) {
                    clone_boxed(&builder, &record)?
                } else {
                    clone_unboxed(context, &builder, &record, &record_type)?
                }),
            )
        },
        fmm_record_type.clone(),
        fmm::types::CallingConvention::Target,
        fmm::ir::Linkage::Weak,
    )?;

    Ok(())
}

fn clone_boxed(
    builder: &fmm::build::InstructionBuilder,
    record: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    pointer::clone(builder, record)
}

fn clone_unboxed(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    record: &fmm::build::TypedExpression,
    record_type: &mir::types::Record,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::record(
        context.types()[record_type.name()]
            .fields()
            .iter()
            .enumerate()
            .map(|(index, type_)| {
                expression::clone(
                    builder,
                    &record::get_unboxed_field(builder, record, index)?,
                    type_,
                    context.types(),
                )
            })
            .collect::<Result<_, _>>()?,
    )
    .into())
}

pub fn compile_drop_function(
    context: &Context,
    definition: &mir::ir::TypeDefinition,
) -> Result<(), CompileError> {
    let record_type = mir::types::Record::new(definition.name());
    let fmm_record_type = type_::compile_record(&record_type, context.types());

    context.module_builder().define_function(
        utilities::get_drop_function_name(definition.name()),
        vec![fmm::ir::Argument::new(
            ARGUMENT_NAME,
            fmm_record_type.clone(),
        )],
        |builder| -> Result<_, CompileError> {
            let record = fmm::build::variable(ARGUMENT_NAME, fmm_record_type.clone());

            if type_::is_record_boxed(&record_type, context.types()) {
                drop_boxed(context, &builder, &record, &record_type)?
            } else {
                drop_unboxed(context, &builder, &record, &record_type)?;
            }

            Ok(builder.return_(fmm::ir::void_value()))
        },
        fmm::types::void_type(),
        fmm::types::CallingConvention::Target,
        fmm::ir::Linkage::Weak,
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
            builder,
            &record::get_unboxed_field(builder, record, index)?,
            type_,
            context.types(),
        )?;
    }

    Ok(())
}

pub fn compile_synchronize_function(
    context: &Context,
    definition: &mir::ir::TypeDefinition,
) -> Result<(), CompileError> {
    let record_type = mir::types::Record::new(definition.name());
    let fmm_record_type = type_::compile_record(&record_type, context.types());

    context.module_builder().define_function(
        utilities::get_synchronize_function_name(definition.name()),
        vec![fmm::ir::Argument::new(
            ARGUMENT_NAME,
            fmm_record_type.clone(),
        )],
        |builder| -> Result<_, CompileError> {
            let record = fmm::build::variable(ARGUMENT_NAME, fmm_record_type.clone());

            if type_::is_record_boxed(&record_type, context.types()) {
                synchronize_boxed(context, &builder, &record, &record_type)?
            } else {
                synchronize_unboxed(context, &builder, &record, &record_type)?;
            }

            Ok(builder.return_(fmm::ir::void_value()))
        },
        fmm::types::void_type(),
        fmm::types::CallingConvention::Target,
        fmm::ir::Linkage::Weak,
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
            builder,
            &record::get_unboxed_field(builder, record, index)?,
            type_,
            context.types(),
        )?;
    }

    Ok(())
}
