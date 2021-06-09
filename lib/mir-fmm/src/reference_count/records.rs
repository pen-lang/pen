use super::{
    super::{error::CompileError, types},
    expressions, pointers, record_utilities,
};
use std::collections::HashMap;

const ARGUMENT_NAME: &str = "_record";

pub fn compile_record_clone_function(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &mir::ir::TypeDefinition,
    types: &HashMap<String, mir::types::RecordBody>,
) -> Result<(), CompileError> {
    let record_type = mir::types::Record::new(definition.name());
    let fmm_record_type = types::compile_record(&record_type, types);

    module_builder.define_function(
        record_utilities::get_record_clone_function_name(definition.name()),
        vec![fmm::ir::Argument::new(
            ARGUMENT_NAME,
            fmm_record_type.clone(),
        )],
        |builder| -> Result<_, CompileError> {
            let record = fmm::build::variable(ARGUMENT_NAME, fmm_record_type.clone());

            if types::is_record_boxed(&record_type, types) {
                pointers::clone_pointer(&builder, &record)?;
            } else {
                for (index, type_) in definition.type_().elements().iter().enumerate() {
                    expressions::clone_expression(
                        &builder,
                        &crate::records::get_record_element(
                            &builder,
                            &record,
                            &record_type,
                            index,
                            types,
                        )?,
                        type_,
                        types,
                    )?;
                }
            }

            Ok(builder.return_(fmm::ir::VOID_VALUE.clone()))
        },
        fmm::types::VOID_TYPE.clone(),
        fmm::types::CallingConvention::Target,
        fmm::ir::Linkage::Weak,
    )?;

    Ok(())
}

pub fn compile_record_drop_function(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &mir::ir::TypeDefinition,
    types: &HashMap<String, mir::types::RecordBody>,
) -> Result<(), CompileError> {
    let record_type = mir::types::Record::new(definition.name());
    let fmm_record_type = types::compile_record(&record_type, types);

    module_builder.define_function(
        record_utilities::get_record_drop_function_name(definition.name()),
        vec![fmm::ir::Argument::new(
            ARGUMENT_NAME,
            fmm_record_type.clone(),
        )],
        |builder| -> Result<_, CompileError> {
            let record = fmm::build::variable(ARGUMENT_NAME, fmm_record_type.clone());

            if types::is_record_boxed(&record_type, types) {
                pointers::drop_pointer(&builder, &record, |builder| {
                    drop_record_elements(
                        &builder,
                        &record,
                        &record_type,
                        definition.type_(),
                        types,
                    )?;

                    Ok(())
                })?;
            } else {
                drop_record_elements(&builder, &record, &record_type, definition.type_(), types)?;
            }

            Ok(builder.return_(fmm::ir::VOID_VALUE.clone()))
        },
        fmm::types::VOID_TYPE.clone(),
        fmm::types::CallingConvention::Target,
        fmm::ir::Linkage::Weak,
    )?;

    Ok(())
}

fn drop_record_elements(
    builder: &fmm::build::InstructionBuilder,
    record: &fmm::build::TypedExpression,
    record_type: &mir::types::Record,
    record_body_type: &mir::types::RecordBody,
    types: &HashMap<String, mir::types::RecordBody>,
) -> Result<(), CompileError> {
    for (index, type_) in record_body_type.elements().iter().enumerate() {
        expressions::drop_expression(
            &builder,
            &crate::records::get_record_element(&builder, &record, &record_type, index, types)?,
            type_,
            types,
        )?;
    }

    Ok(())
}
