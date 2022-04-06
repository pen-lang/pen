use super::error::CompileError;
use crate::{closures, context::Context, entry_functions, types};
use fnv::FnvHashMap;

pub fn compile_function_definition(
    context: &Context,
    definition: &mir::ir::Definition,
    global_variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<(), CompileError> {
    context.module_builder().define_variable(
        definition.name(),
        closures::compile_closure_content(
            entry_functions::compile(context, definition, true, global_variables)?,
            closures::compile_drop_function(context, definition)?,
            fmm::ir::Undefined::new(types::compile_closure_payload(definition, context.types())),
        ),
        definition.is_thunk(),
        fmm::ir::Linkage::External,
        None,
    );

    Ok(())
}
