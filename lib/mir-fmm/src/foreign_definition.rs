use crate::{call, context::Context, type_, CompileError};

pub fn compile_foreign_definition(
    context: &Context,
    definition: &mir::ir::ForeignDefinition,
    function_type: &mir::types::Function,
    global_variable: &fmm::build::TypedExpression,
) -> Result<(), CompileError> {
    let foreign_function_type = type_::compile_foreign_function(
        function_type,
        definition.calling_convention(),
        context.types(),
    );
    let arguments = foreign_function_type
        .arguments()
        .iter()
        .enumerate()
        .map(|(index, type_)| fmm::ir::Argument::new(format!("arg_{}", index), type_.clone()))
        .collect::<Vec<_>>();

    context.module_builder().define_function(
        definition.foreign_name(),
        arguments.clone(),
        |instruction_builder| -> Result<_, CompileError> {
            Ok(instruction_builder.return_(call::compile(
                &instruction_builder,
                global_variable,
                &arguments
                    .iter()
                    .map(|argument| fmm::build::variable(argument.name(), argument.type_().clone()))
                    .collect::<Vec<_>>(),
            )?))
        },
        foreign_function_type.result().clone(),
        foreign_function_type.calling_convention(),
        fmm::ir::Linkage::External,
    )?;

    Ok(())
}
