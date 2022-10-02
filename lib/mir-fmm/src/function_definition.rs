use crate::{
    closure, context::Context, entry_function, error::CompileError, reference_count, type_,
};
use fnv::FnvHashMap;

pub fn compile(
    context: &Context,
    definition: &mir::ir::GlobalFunctionDefinition,
    global_variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<(), CompileError> {
    let public = definition.is_public();
    let definition = definition.definition();

    context.module_builder().define_variable(
        definition.name(),
        reference_count::block::compile_static(closure::compile_content(
            entry_function::compile(context, definition, true, global_variables)?,
            closure::metadata::compile(context, definition)?,
            fmm::ir::Undefined::new(type_::compile_closure_payload(context, definition)),
        ))?,
        fmm::ir::VariableDefinitionOptions::new()
            .set_address_named(false)
            .set_linkage(if public {
                fmm::ir::Linkage::External
            } else {
                fmm::ir::Linkage::Internal
            })
            .set_mutable(definition.is_thunk()),
    );

    Ok(())
}
