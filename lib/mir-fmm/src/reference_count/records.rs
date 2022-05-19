use super::{
    super::{error::CompileError, types},
    expressions, pointers, record_utilities,
};
use crate::context::Context;
use fnv::FnvHashMap;

const ARGUMENT_NAME: &str = "_record";

pub fn compile_record_clone_function(
    context: &Context,
    definition: &mir::ir::TypeDefinition,
) -> Result<(), CompileError> {
    let record_type = mir::types::Record::new(definition.name());
    let fmm_record_type = types::compile_record(&record_type, context.types());

    context.module_builder().define_function(
        record_utilities::get_record_clone_function_name(definition.name()),
        vec![fmm::ir::Argument::new(
            ARGUMENT_NAME,
            fmm_record_type.clone(),
        )],
        |builder| -> Result<_, CompileError> {
            let record = fmm::build::variable(ARGUMENT_NAME, fmm_record_type.clone());

            Ok(
                builder.return_(if types::is_record_boxed(&record_type, context.types()) {
                    pointers::clone_pointer(&builder, &record)?
                } else {
                    fmm::build::record(
                        definition
                            .type_()
                            .fields()
                            .iter()
                            .enumerate()
                            .map(|(index, type_)| {
                                expressions::clone_expression(
                                    &builder,
                                    &crate::records::get_record_field(
                                        &builder,
                                        &record,
                                        &record_type,
                                        index,
                                        context.types(),
                                    )?,
                                    type_,
                                    context.types(),
                                )
                            })
                            .collect::<Result<_, _>>()?,
                    )
                    .into()
                }),
            )
        },
        fmm_record_type.clone(),
        fmm::types::CallingConvention::Target,
        fmm::ir::Linkage::Internal,
    )?;

    Ok(())
}

pub fn compile_record_drop_function(
    context: &Context,
    definition: &mir::ir::TypeDefinition,
) -> Result<(), CompileError> {
    let record_type = mir::types::Record::new(definition.name());
    let fmm_record_type = types::compile_record(&record_type, context.types());

    context.module_builder().define_function(
        record_utilities::get_record_drop_function_name(definition.name()),
        vec![fmm::ir::Argument::new(
            ARGUMENT_NAME,
            fmm_record_type.clone(),
        )],
        |builder| -> Result<_, CompileError> {
            let record = fmm::build::variable(ARGUMENT_NAME, fmm_record_type.clone());

            if types::is_record_boxed(&record_type, context.types()) {
                pointers::drop_pointer(&builder, &record, |builder| {
                    drop_record_fields(builder, &record, &record_type, context.types())
                })?;
            } else {
                drop_record_fields(&builder, &record, &record_type, context.types())?;
            }

            Ok(builder.return_(fmm::ir::VOID_VALUE.clone()))
        },
        fmm::types::VOID_TYPE.clone(),
        fmm::types::CallingConvention::Target,
        fmm::ir::Linkage::Internal,
    )?;

    Ok(())
}

pub fn compile_record_drop_or_reuse_function(
    context: &Context,
    definition: &mir::ir::TypeDefinition,
) -> Result<(), CompileError> {
    let record_type = mir::types::Record::new(definition.name());

    if !types::is_record_boxed(&record_type, context.types()) {
        return Ok(());
    }

    let fmm_record_type = types::compile_record(&record_type, context.types());

    context.module_builder().define_function(
        record_utilities::get_record_drop_or_reuse_function_name(definition.name()),
        vec![fmm::ir::Argument::new(
            ARGUMENT_NAME,
            fmm_record_type.clone(),
        )],
        |builder| -> Result<_, CompileError> {
            let record = fmm::build::variable(ARGUMENT_NAME, fmm_record_type.clone());

            Ok(builder.return_(pointers::drop_or_reuse_pointer(
                &builder,
                &record,
                |builder| drop_record_fields(builder, &record, &record_type, context.types()),
            )?))
        },
        fmm_record_type.clone(),
        fmm::types::CallingConvention::Target,
        fmm::ir::Linkage::Internal,
    )?;

    Ok(())
}

fn drop_record_fields(
    builder: &fmm::build::InstructionBuilder,
    record: &fmm::build::TypedExpression,
    record_type: &mir::types::Record,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<(), CompileError> {
    for (index, type_) in types[record_type.name()].fields().iter().enumerate() {
        expressions::drop_expression(
            builder,
            &crate::records::get_record_field(builder, record, record_type, index, types)?,
            type_,
            types,
        )?;
    }

    Ok(())
}
