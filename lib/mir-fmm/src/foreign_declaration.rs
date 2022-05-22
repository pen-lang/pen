use crate::{
    closure,
    context::Context,
    type_::{self, FUNCTION_ARGUMENT_OFFSET},
};

pub fn compile_foreign_declaration(
    context: &Context,
    declaration: &mir::ir::ForeignDeclaration,
) -> Result<(), fmm::build::BuildError> {
    context.module_builder().define_variable(
        declaration.name(),
        closure::compile_closure_content(
            compile_entry_function(context, declaration)?,
            fmm::ir::Undefined::new(type_::compile_closure_drop_function()),
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
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
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

    let foreign_function_type = type_::compile_foreign_function(
        declaration.type_(),
        declaration.calling_convention(),
        context.types(),
    );

    context.module_builder().define_anonymous_function(
        arguments.clone(),
        |instruction_builder| {
            Ok(instruction_builder.return_(
                instruction_builder.call(
                    context.module_builder().declare_function(
                        declaration.foreign_name(),
                        foreign_function_type.clone(),
                    ),
                    arguments
                        .iter()
                        .skip(FUNCTION_ARGUMENT_OFFSET)
                        .map(|argument| {
                            fmm::build::variable(argument.name(), argument.type_().clone())
                        })
                        .collect(),
                )?,
            ))
        },
        foreign_function_type.result().clone(),
        fmm::types::CallingConvention::Source,
    )
}
