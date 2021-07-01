mod compile_configuration;

use crate::{
    common::{dependency_serializer, interface_serializer},
    infra::{FilePath, Infrastructure},
    prelude_interface_file_finder,
};
pub use compile_configuration::{
    CompileConfiguration, HeapConfiguration, ListTypeConfiguration, StringTypeConfiguration,
};
use std::error::Error;

const PRELUDE_PREFIX: &str = "prelude:";

#[allow(clippy::too_many_arguments)]
pub fn compile(
    infrastructure: &Infrastructure,
    source_file: &FilePath,
    dependency_file: &FilePath,
    object_file: &FilePath,
    interface_file: &FilePath,
    compile_configuration: &CompileConfiguration,
    output_directory: &FilePath,
    prelude_package_url: &url::Url,
) -> Result<(), Box<dyn Error>> {
    let dependencies = dependency_serializer::deserialize(
        &infrastructure.file_system.read_to_vec(dependency_file)?,
    )?;

    let ast_module = lang::parse::parse(
        &infrastructure.file_system.read_to_string(source_file)?,
        &infrastructure.file_path_displayer.display(source_file),
    )?;

    let (module, module_interface) = lang::hir_mir::compile(
        &lang::ast_hir::compile(
            &ast_module,
            &format!(
                "{}:",
                infrastructure.file_path_displayer.display(source_file)
            ),
            &ast_module
                .imports()
                .iter()
                .map(|import| {
                    Ok((
                        import.module_path().clone(),
                        interface_serializer::deserialize(
                            &infrastructure
                                .file_system
                                .read_to_vec(&dependencies[import.module_path()].clone())?,
                        )?,
                    ))
                })
                .collect::<Result<_, Box<dyn Error>>>()?,
            &prelude_interface_file_finder::find(
                infrastructure,
                output_directory,
                prelude_package_url,
            )?
            .iter()
            .map(|file| {
                interface_serializer::deserialize(&infrastructure.file_system.read_to_vec(file)?)
            })
            .collect::<Result<Vec<_>, _>>()?,
        )?,
        &compile_configuration.list_type,
        &compile_configuration.string_type,
    )?;

    compile_mir_module(infrastructure, &module, object_file, compile_configuration)?;
    infrastructure.file_system.write(
        interface_file,
        &interface_serializer::serialize(&module_interface)?,
    )?;

    Ok(())
}

pub fn compile_prelude(
    infrastructure: &Infrastructure,
    source_file: &FilePath,
    object_file: &FilePath,
    interface_file: &FilePath,
    compile_configuration: &CompileConfiguration,
) -> Result<(), Box<dyn Error>> {
    // TODO Implement and use lang::hir_mir::compile_prelude().
    let (module, module_interface) = lang::hir_mir::compile(
        &lang::ast_hir::compile_prelude(
            &lang::parse::parse(
                &infrastructure.file_system.read_to_string(source_file)?,
                &infrastructure.file_path_displayer.display(source_file),
            )?,
            PRELUDE_PREFIX,
        )?,
        &compile_configuration.list_type,
        &compile_configuration.string_type,
    )?;

    compile_mir_module(infrastructure, &module, object_file, compile_configuration)?;
    infrastructure.file_system.write(
        interface_file,
        &interface_serializer::serialize(&module_interface)?,
    )?;

    Ok(())
}

fn compile_mir_module(
    infrastructure: &Infrastructure,
    module: &mir::ir::Module,
    object_file: &FilePath,
    compile_configuration: &CompileConfiguration,
) -> Result<(), Box<dyn Error>> {
    infrastructure.file_system.write(
        object_file,
        &fmm_llvm::compile_to_bit_code(
            &fmm::analysis::transform_to_cps(
                &mir_fmm::compile(module)?,
                fmm::types::VOID_TYPE.clone(),
            )?,
            &compile_configuration.heap,
            None,
        )?,
    )?;

    Ok(())
}
