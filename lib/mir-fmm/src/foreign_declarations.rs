use crate::{
    closures,
    types::{self, FUNCTION_ARGUMENT_OFFSET},
};
use std::collections::BTreeMap;

pub fn compile_foreign_declaration(
    module_builder: &fmm::build::ModuleBuilder,
    declaration: &mir::ir::ForeignDeclaration,
    types: &BTreeMap<String, mir::types::RecordBody>,
) -> Result<(), fmm::build::BuildError> {
    module_builder.define_variable(
        declaration.name(),
        closures::compile_closure_content(
            compile_entry_function(module_builder, declaration, types)?,
            fmm::ir::Undefined::new(types::compile_closure_drop_function()),
            fmm::build::record(vec![]),
        ),
        false,
        fmm::ir::Linkage::Internal,
        None,
    );

    Ok(())
}

fn compile_entry_function(
    module_builder: &fmm::build::ModuleBuilder,
    declaration: &mir::ir::ForeignDeclaration,
    types: &BTreeMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    let arguments = vec![fmm::ir::Argument::new(
        "_closure",
        types::compile_untyped_closure_pointer(),
    )]
    .into_iter()
    .chain(
        declaration
            .type_()
            .arguments()
            .iter()
            .enumerate()
            .map(|(index, type_)| {
                fmm::ir::Argument::new(format!("arg_{}", index), types::compile(type_, types))
            }),
    )
    .collect::<Vec<_>>();

    let foreign_function_type = types::compile_foreign_function(
        declaration.type_(),
        declaration.calling_convention(),
        types,
    );

    module_builder.define_anonymous_function(
        arguments.clone(),
        |instruction_builder| {
            Ok(instruction_builder.return_(
                instruction_builder.call(
                    module_builder.declare_function(
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
