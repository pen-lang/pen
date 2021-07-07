mod compile_configuration;
mod main_module_configuration_qualifier;
mod prelude_type_configuration_qualifier;
mod utilities;

use crate::{
    application_configuration::ApplicationConfiguration,
    common::{dependency_serializer, file_path_resolver, interface_serializer},
    infra::{FilePath, Infrastructure},
    prelude_interface_file_finder,
};
pub use compile_configuration::{
    CompileConfiguration, HeapConfiguration, ListTypeConfiguration, StringTypeConfiguration,
};
use std::error::Error;

use self::utilities::{DUMMY_LIST_TYPE_CONFIGURATION, DUMMY_STRING_TYPE_CONFIGURATION};

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
    let (module, module_interface) = lang::hir_mir::compile(
        &compile_to_hir(
            infrastructure,
            source_file,
            dependency_file,
            output_directory,
            prelude_package_url,
        )?,
        &prelude_type_configuration_qualifier::qualify_list_type_configuration(
            &compile_configuration.list_type,
            PRELUDE_PREFIX,
        ),
        &prelude_type_configuration_qualifier::qualify_string_type_configuration(
            &compile_configuration.string_type,
            PRELUDE_PREFIX,
        ),
    )?;

    compile_mir_module(
        infrastructure,
        &module,
        object_file,
        &compile_configuration.heap,
    )?;
    infrastructure.file_system.write(
        interface_file,
        &interface_serializer::serialize(&module_interface)?,
    )?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn compile_main(
    infrastructure: &Infrastructure,
    source_file: &FilePath,
    dependency_file: &FilePath,
    object_file: &FilePath,
    system_package_directory: &FilePath,
    compile_configuration: &CompileConfiguration,
    output_directory: &FilePath,
    prelude_package_url: &url::Url,
    application_configuration: &ApplicationConfiguration,
) -> Result<(), Box<dyn Error>> {
    compile_mir_module(
        infrastructure,
        &lang::hir_mir::compile_main(
            &compile_to_hir(
                infrastructure,
                source_file,
                dependency_file,
                output_directory,
                prelude_package_url,
            )?,
            &prelude_type_configuration_qualifier::qualify_list_type_configuration(
                &compile_configuration.list_type,
                PRELUDE_PREFIX,
            ),
            &prelude_type_configuration_qualifier::qualify_string_type_configuration(
                &compile_configuration.string_type,
                PRELUDE_PREFIX,
            ),
            &main_module_configuration_qualifier::qualify(
                &application_configuration.main_module,
                &calculate_module_prefix(
                    infrastructure,
                    &file_path_resolver::resolve_source_file(
                        system_package_directory,
                        &[application_configuration
                            .main_function_module_basename
                            .clone()],
                        &infrastructure.file_path_configuration,
                    ),
                ),
            ),
        )?,
        object_file,
        &compile_configuration.heap,
    )?;

    Ok(())
}

fn compile_to_hir(
    infrastructure: &Infrastructure,
    source_file: &FilePath,
    dependency_file: &FilePath,
    output_directory: &FilePath,
    prelude_package_url: &url::Url,
) -> Result<lang::hir::Module, Box<dyn Error>> {
    let dependencies = dependency_serializer::deserialize(
        &infrastructure.file_system.read_to_vec(dependency_file)?,
    )?;

    let ast_module = lang::parse::parse(
        &infrastructure.file_system.read_to_string(source_file)?,
        &infrastructure.file_path_displayer.display(source_file),
    )?;

    Ok(lang::ast_hir::compile(
        &ast_module,
        &calculate_module_prefix(infrastructure, source_file),
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
    )?)
}

pub fn compile_prelude(
    infrastructure: &Infrastructure,
    source_file: &FilePath,
    object_file: &FilePath,
    interface_file: &FilePath,
    heap_configuration: &HeapConfiguration,
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
        &DUMMY_LIST_TYPE_CONFIGURATION,
        &DUMMY_STRING_TYPE_CONFIGURATION,
    )?;

    compile_mir_module(infrastructure, &module, object_file, heap_configuration)?;
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
    heap_configuration: &HeapConfiguration,
) -> Result<(), Box<dyn Error>> {
    infrastructure.file_system.write(
        object_file,
        &fmm_llvm::compile_to_bit_code(
            &fmm::analysis::transform_to_cps(
                &mir_fmm::compile(module)?,
                fmm::types::VOID_TYPE.clone(),
            )?,
            heap_configuration,
            None,
        )?,
    )?;

    Ok(())
}

fn calculate_module_prefix(infrastructure: &Infrastructure, source_file: &FilePath) -> String {
    format!(
        "{}:",
        infrastructure.file_path_displayer.display(source_file)
    )
}
