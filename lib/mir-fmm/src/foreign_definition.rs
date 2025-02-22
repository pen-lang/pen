use crate::{CompileError, call, context::Context, foreign_value, type_};

pub fn compile(
    context: &Context,
    definition: &mir::ir::ForeignDefinition,
    function_type: &mir::types::Function,
    global_variable: &fmm::build::TypedExpression,
) -> Result<(), CompileError> {
    let foreign_function_type =
        type_::foreign::compile_function(context, function_type, definition.calling_convention())?;
    let arguments = foreign_function_type
        .arguments()
        .iter()
        .enumerate()
        .map(|(index, type_)| fmm::ir::Argument::new(format!("arg_{index}"), type_.clone()))
        .collect::<Vec<_>>();

    context.module_builder().define_function(
        definition.foreign_name(),
        arguments.clone(),
        foreign_function_type.result().clone(),
        |builder| -> Result<_, CompileError> {
            Ok(builder.return_(foreign_value::convert_to_foreign(
                context,
                &builder,
                call::compile(
                    &builder,
                    global_variable,
                    &arguments
                        .iter()
                        .zip(function_type.arguments())
                        .map(|(argument, type_)| {
                            foreign_value::convert_from_foreign(
                                context,
                                &builder,
                                fmm::build::variable(argument.name(), argument.type_().clone()),
                                type_,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                )?,
                function_type.result(),
            )?))
        },
        fmm::ir::FunctionDefinitionOptions::new()
            .set_address_named(false)
            .set_calling_convention(foreign_function_type.calling_convention())
            .set_linkage(fmm::ir::Linkage::External),
    )?;

    Ok(())
}
