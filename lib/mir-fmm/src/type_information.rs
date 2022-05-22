use crate::{context::Context, error::CompileError, reference_count, type_};

pub const TYPE_INFORMATION_CLONE_FUNCTION_FIELD_INDEX: usize = 0;
pub const TYPE_INFORMATION_DROP_FUNCTION_FIELD_INDEX: usize = 1;

pub fn compile_type_information_global_variable(
    context: &Context,
    type_: &mir::types::Type,
) -> Result<(), CompileError> {
    context.module_builder().define_variable(
        type_::compile_type_id(type_),
        fmm::build::record(vec![
            reference_count::variant::compile_clone_function(context, type_)?,
            reference_count::variant::compile_drop_function(context, type_)?,
        ]),
        false,
        fmm::ir::Linkage::Weak,
        None,
    );

    Ok(())
}
