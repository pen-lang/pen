use crate::{
    closure,
    context::Context,
    foreign_value,
    type_::{self, FUNCTION_ARGUMENT_OFFSET},
    CompileError,
};

pub fn compile(
    context: &Context,
    declaration: &mir::ir::ForeignDeclaration,
) -> Result<(), CompileError> {
    context.module_builder().define_variable(
        declaration.name(),
        closure::compile_content(
            compile_entry_function(context, declaration)?,
            fmm::ir::Undefined::new(type_::compile_closure_metadata()),
            fmm::build::record(vec![]),
        ),
        false,
        fmm::ir::Linkage::Internal,
        None,
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
                fmm::ir::Argument::new(
                    format!("arg_{}", index),
                    type_::compile(type_, context.types()),
                )
            }),
    )
    .collect::<Vec<_>>();

    context.module_builder().define_anonymous_function(
        arguments.clone(),
        |instruction_builder| {
            Ok(
                instruction_builder.return_(foreign_value::convert_from_foreign(
                    &instruction_builder,
                    instruction_builder.call(
                        context.module_builder().declare_function(
                            declaration.foreign_name(),
                            type_::foreign::compile_function(
                                declaration.type_(),
                                declaration.calling_convention(),
                                context.types(),
                            )?,
                        ),
                        arguments[FUNCTION_ARGUMENT_OFFSET..]
                            .iter()
                            .zip(declaration.type_().arguments())
                            .map(|(argument, type_)| {
                                foreign_value::convert_to_foreign(
                                    &instruction_builder,
                                    fmm::build::variable(argument.name(), argument.type_().clone()),
                                    type_,
                                    context.types(),
                                )
                            })
                            .collect::<Result<_, _>>()?,
                    )?,
                    declaration.type_().result(),
                    context.types(),
                )?),
            )
        },
        type_::compile(declaration.type_().result(), context.types()),
        fmm::types::CallingConvention::Source,
    )
}
