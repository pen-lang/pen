use crate::{context::Context, error::CompileError, reference_count};
use std::collections::HashMap;

pub fn compile_global_variable(
    context: &Context,
    type_: &mir::types::Type,
    global_variables: &HashMap<String, fmm::build::TypedExpression>,
) -> Result<(), CompileError> {
    context.module_builder().define_variable(
        get_global_variable_name(type_),
        fmm::build::record(vec![
            reference_count::variant::compile_clone_function(context, type_)?,
            reference_count::variant::compile_drop_function(context, type_)?,
            reference_count::variant::compile_synchronize_function(context, type_)?,
            fmm::build::bit_cast(
                fmm::types::generic_pointer_type(),
                global_variables[context
                    .type_information()
                    .information()
                    .get(type_)
                    .ok_or_else(|| CompileError::TypeInformationNotFound(type_.clone()))?]
                .clone(),
            )
            .into(),
        ]),
        fmm::ir::VariableDefinitionOptions::new()
            .set_address_named(true)
            .set_linkage(fmm::ir::Linkage::Weak)
            .set_mutable(false),
    );

    Ok(())
}

pub fn get_global_variable_name(type_: &mir::types::Type) -> String {
    format!(
        "mir:type_information:{}",
        mir::analysis::type_id::calculate(type_),
    )
}

pub fn get_clone_function(
    builder: &fmm::build::InstructionBuilder,
    tag: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    get_field(builder, tag, 0)
}

pub fn get_drop_function(
    builder: &fmm::build::InstructionBuilder,
    tag: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    get_field(builder, tag, 1)
}

pub fn get_synchronize_function(
    builder: &fmm::build::InstructionBuilder,
    tag: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    get_field(builder, tag, 2)
}

pub fn get_custom_information(
    builder: &fmm::build::InstructionBuilder,
    tag: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    get_field(builder, tag, 3)
}

fn get_field(
    builder: &fmm::build::InstructionBuilder,
    tag: impl Into<fmm::build::TypedExpression>,
    index: usize,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(builder.deconstruct_record(builder.load(tag)?, index)?)
}
