use super::error::CompileError;
use crate::{closure, context::Context, entry_function, types};
use fnv::FnvHashMap;

pub fn compile_function_definition(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
    global_variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<(), CompileError> {
    context.module_builder().define_variable(
        definition.name(),
        closure::compile_closure_content(
            entry_function::compile(context, definition, true, global_variables)?,
            closure::compile_drop_function(context, definition)?,
            fmm::ir::Undefined::new(types::compile_closure_payload(definition, context.types())),
        ),
        definition.is_thunk(),
        fmm::ir::Linkage::External,
        None,
    );

    Ok(())
}
