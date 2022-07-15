use crate::{
    closure, context::Context, entry_function, error::CompileError, reference_count, type_,
};
use fnv::FnvHashMap;

pub fn compile(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
    global_variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<(), CompileError> {
    context.module_builder().define_variable(
        definition.name(),
        fmm::build::record(vec![
            reference_count::count::compile_static()?,
            closure::compile_content(
                entry_function::compile(context, definition, true, global_variables)?,
                closure::metadata::compile(context, definition)?,
                fmm::ir::Undefined::new(type_::compile_closure_payload(
                    definition,
                    context.types(),
                )),
            ),
        ]),
        definition.is_thunk(),
        fmm::ir::Linkage::External,
        None,
    );

    Ok(())
}
