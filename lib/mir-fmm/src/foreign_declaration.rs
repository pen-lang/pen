use crate::{
    closure,
    context::Context,
    foreign_value, reference_count,
    type_::{self, FUNCTION_ARGUMENT_OFFSET},
    CompileError,
};

pub fn compile(
    context: &Context,
    declaration: &mir::ir::ForeignDeclaration,
) -> Result<(), CompileError> {
    context.module_builder().define_variable(
        declaration.name(),
        reference_count::block::compile_static(closure::compile_content(
            compile_entry_function(context, declaration)?,
            fmm::ir::Undefined::new(type_::compile_closure_metadata()),
            fmm::build::record(vec![]),
        ))?,
        fmm::ir::VariableDefinitionOptions::new()
            .set_address_named(false)
            .set_linkage(fmm::ir::Linkage::Internal)
            .set_mutable(false),
    );

    Ok(())
}

fn compile_entry_function(
    context: &Context,
    declaration: &mir::ir::ForeignDeclaration,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let arguments = [fmm::ir::Argument::new(
        "_closure",
        type_::compile_untyped_closure_pointer(),
    )]
    .into_iter()
    .chain(
        declaration
            .type_()
            .arguments()
            .iter()
            .enumerate()
            .map(|(index, type_)| {
                fmm::ir::Argument::new(format!("arg_{index}"), type_::compile(context, type_))
            }),
    )
    .collect::<Vec<_>>();

    context.module_builder().define_anonymous_function(
        arguments.clone(),
        type_::compile(context, declaration.type_().result()),
        |instruction_builder| {
            Ok(
                instruction_builder.return_(foreign_value::convert_from_foreign(
                    context,
                    &instruction_builder,
                    instruction_builder.call(
                        context.module_builder().declare_function(
                            declaration.foreign_name(),
                            type_::foreign::compile_function(
                                context,
                                declaration.type_(),
                                declaration.calling_convention(),
                            )?,
                        ),
                        arguments[FUNCTION_ARGUMENT_OFFSET..]
                            .iter()
                            .zip(declaration.type_().arguments())
                            .map(|(argument, type_)| {
                                foreign_value::convert_to_foreign(
                                    context,
                                    &instruction_builder,
                                    fmm::build::variable(argument.name(), argument.type_().clone()),
                                    type_,
                                )
                            })
                            .collect::<Result<_, _>>()?,
                    )?,
                    declaration.type_().result(),
                )?),
            )
        },
        fmm::ir::FunctionDefinitionOptions::new()
            .set_address_named(false)
            .set_calling_convention(fmm::types::CallingConvention::Source)
            .set_linkage(fmm::ir::Linkage::Internal),
    )
}
