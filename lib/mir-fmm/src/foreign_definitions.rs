use crate::{function_applications, types, CompileError};
use std::collections::HashMap;

pub fn compile_foreign_definition(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &mir::ir::ForeignDefinition,
    function_type: &mir::types::Function,
    global_variable: &fmm::build::TypedExpression,
    types: &HashMap<String, mir::types::RecordBody>,
) -> Result<(), CompileError> {
    // TODO Support a target calling convention.
    // Blocked by https://github.com/raviqqe/fmm/issues/88
    let foreign_function_type =
        types::compile_foreign_function(function_type, mir::ir::CallingConvention::Source, types);
    let arguments = foreign_function_type
        .arguments()
        .iter()
        .enumerate()
        .map(|(index, type_)| fmm::ir::Argument::new(format!("arg_{}", index), type_.clone()))
        .collect::<Vec<_>>();

    module_builder.define_function(
        definition.foreign_name(),
        arguments.clone(),
        |instruction_builder| -> Result<_, CompileError> {
            Ok(instruction_builder.return_(function_applications::compile(
                module_builder,
                &instruction_builder,
                global_variable.clone(),
                &arguments
                    .iter()
                    .map(|argument| fmm::build::variable(argument.name(), argument.type_().clone()))
                    .collect::<Vec<_>>(),
                &function_type.arguments().into_iter().collect::<Vec<_>>(),
                types,
            )?))
        },
        foreign_function_type.result().clone(),
        foreign_function_type.calling_convention(),
        fmm::ir::Linkage::External,
    )?;

    Ok(())
}
