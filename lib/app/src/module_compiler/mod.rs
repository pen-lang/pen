mod compile_configuration;
mod error;
mod main_module_configuration_qualifier;
mod prelude_type_configuration_qualifier;

use crate::{
    application_configuration::ApplicationConfiguration,
    common::{dependency_serializer, interface_serializer},
    infra::{FilePath, Infrastructure},
};
pub use compile_configuration::{
    CompileConfiguration, ErrorTypeConfiguration, HeapConfiguration, ListTypeConfiguration,
    StringTypeConfiguration,
};
use std::error::Error;

const PRELUDE_PREFIX: &str = "prelude:";

pub fn compile(
    infrastructure: &Infrastructure,
    source_file: &FilePath,
    dependency_file: &FilePath,
    object_file: &FilePath,
    interface_file: &FilePath,
    compile_configuration: &CompileConfiguration,
) -> Result<(), Box<dyn Error>> {
    let (module, module_interface) = lang::hir_mir::compile(
        &compile_to_hir(infrastructure, source_file, dependency_file, None)?,
        &prelude_type_configuration_qualifier::qualify_list_type_configuration(
            &compile_configuration.list_type,
            PRELUDE_PREFIX,
        ),
        &prelude_type_configuration_qualifier::qualify_string_type_configuration(
            &compile_configuration.string_type,
            PRELUDE_PREFIX,
        ),
        &prelude_type_configuration_qualifier::qualify_error_type_configuration(
            &compile_configuration.error_type,
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
    main_function_interface_file: &FilePath,
    compile_configuration: &CompileConfiguration,
    application_configuration: &ApplicationConfiguration,
) -> Result<(), Box<dyn Error>> {
    let main_function_interface = interface_serializer::deserialize(
        &infrastructure
            .file_system
            .read_to_vec(main_function_interface_file)?,
    )?;

    compile_mir_module(
        infrastructure,
        &lang::hir_mir::compile_main(
            &compile_to_hir(
                infrastructure,
                source_file,
                dependency_file,
                Some(&main_function_interface),
            )?,
            &prelude_type_configuration_qualifier::qualify_list_type_configuration(
                &compile_configuration.list_type,
                PRELUDE_PREFIX,
            ),
            &prelude_type_configuration_qualifier::qualify_string_type_configuration(
                &compile_configuration.string_type,
                PRELUDE_PREFIX,
            ),
            &prelude_type_configuration_qualifier::qualify_error_type_configuration(
                &compile_configuration.error_type,
                PRELUDE_PREFIX,
            ),
            &main_module_configuration_qualifier::qualify(
                &application_configuration.main_module,
                &main_function_interface,
            )?,
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
    main_function_interface: Option<&lang::interface::Module>,
) -> Result<lang::hir::Module, Box<dyn Error>> {
    let (interface_files, prelude_interface_files) = dependency_serializer::deserialize(
        &infrastructure.file_system.read_to_vec(dependency_file)?,
    )?;

    let ast_module = lang::parse::parse(
        &infrastructure.file_system.read_to_string(source_file)?,
        &infrastructure.file_path_displayer.display(source_file),
    )?;

    Ok(lang::ast_hir::compile(
        &ast_module,
        &calculate_module_prefix(source_file),
        &ast_module
            .imports()
            .iter()
            .map(|import| {
                Ok((
                    import.module_path().clone(),
                    interface_serializer::deserialize(
                        &infrastructure
                            .file_system
                            .read_to_vec(&interface_files[import.module_path()].clone())?,
                    )?,
                ))
            })
            .collect::<Result<_, Box<dyn Error>>>()?,
        &prelude_interface_files
            .iter()
            .map(|file| {
                interface_serializer::deserialize(&infrastructure.file_system.read_to_vec(file)?)
            })
            .chain(main_function_interface.cloned().map(Ok))
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
    let (module, module_interface) =
        lang::hir_mir::compile_prelude(&lang::ast_hir::compile_prelude(
            &lang::parse::parse(
                &infrastructure.file_system.read_to_string(source_file)?,
                &infrastructure.file_path_displayer.display(source_file),
            )?,
            PRELUDE_PREFIX,
        )?)?;

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

fn calculate_module_prefix(source_file: &FilePath) -> String {
    format!("{}:", source_file)
}
