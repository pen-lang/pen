use crate::{call, context::Context, foreign_value, type_, CompileError};

pub fn compile_foreign_definition(
    context: &Context,
    definition: &mir::ir::ForeignDefinition,
    function_type: &mir::types::Function,
    global_variable: &fmm::build::TypedExpression,
) -> Result<(), CompileError> {
    let foreign_function_type = type_::foreign::compile_function(
        function_type,
        definition.calling_convention(),
        context.types(),
    )?;
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
            Ok(
                instruction_builder.return_(foreign_value::convert_to_foreign(
                    &instruction_builder,
                    call::compile(
                        &instruction_builder,
                        global_variable,
                        &arguments
                            .iter()
                            .zip(function_type.arguments())
                            .map(|(argument, type_)| {
                                foreign_value::convert_from_foreign(
                                    &instruction_builder,
                                    fmm::build::variable(argument.name(), argument.type_().clone()),
                                    type_,
                                    context.types(),
                                )
                            })
                            .collect::<Result<Vec<_>, _>>()?,
                    )?,
                    function_type.result(),
                    context.types(),
                )?),
            )
        },
        foreign_function_type.result().clone(),
        foreign_function_type.calling_convention(),
        fmm::ir::Linkage::External,
    )?;

    Ok(())
}
