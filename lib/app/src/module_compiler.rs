mod compile_configuration;
mod main_module_configuration_qualifier;
mod prelude_type_configuration_qualifier;

use crate::{
    application_configuration::ApplicationConfiguration,
    common::{dependency_serializer, interface_serializer, module_test_information_serializer},
    infra::{FilePath, Infrastructure},
    test_configuration::TestModuleConfiguration,
};
pub use compile_configuration::{
    CompileConfiguration, ErrorTypeConfiguration, FmmConfiguration, HashConfiguration,
    HirConfiguration, ListTypeConfiguration, MapTypeConfiguration, MapTypeIterationConfiguration,
    MirConfiguration, StringTypeConfiguration,
};
use fnv::FnvHashMap;
use std::{collections::BTreeMap, error::Error};

const PRELUDE_PREFIX: &str = "prelude:";

pub fn compile(
    infrastructure: &Infrastructure,
    source_file: &FilePath,
    dependency_file: &FilePath,
    object_file: &FilePath,
    interface_file: &FilePath,
    target_triple: Option<&str>,
    compile_configuration: &CompileConfiguration,
) -> Result<(), Box<dyn Error>> {
    let (module, module_interface) = hir_mir::compile(
        &compile_to_hir(infrastructure, source_file, dependency_file, &[])?,
        &prelude_type_configuration_qualifier::qualify(&compile_configuration.hir, PRELUDE_PREFIX),
    )?;

    compile_mir_module(
        infrastructure,
        &module,
        object_file,
        target_triple,
        compile_configuration,
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
    context_interface_files: &BTreeMap<String, FilePath>,
    target_triple: Option<&str>,
    compile_configuration: &CompileConfiguration,
    application_configuration: &ApplicationConfiguration,
) -> Result<(), Box<dyn Error>> {
    let context_interfaces = context_interface_files
        .iter()
        .map(|(key, file)| {
            Ok((
                key.clone(),
                interface_serializer::deserialize(&infrastructure.file_system.read_to_vec(file)?)?,
            ))
        })
        .collect::<Result<FnvHashMap<_, _>, Box<dyn Error>>>()?;

    compile_mir_module(
        infrastructure,
        &hir_mir::compile_main(
            &compile_to_hir(
                infrastructure,
                source_file,
                dependency_file,
                &context_interfaces.values().cloned().collect::<Vec<_>>(),
            )?,
            &prelude_type_configuration_qualifier::qualify(
                &compile_configuration.hir,
                PRELUDE_PREFIX,
            ),
            &main_module_configuration_qualifier::qualify(
                &application_configuration.main_module,
                &context_interfaces,
            )?,
        )?,
        object_file,
        target_triple,
        compile_configuration,
    )?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn compile_test(
    infrastructure: &Infrastructure,
    source_file: &FilePath,
    dependency_file: &FilePath,
    object_file: &FilePath,
    test_information_file: &FilePath,
    target_triple: Option<&str>,
    compile_configuration: &CompileConfiguration,
    test_module_configuration: &TestModuleConfiguration,
) -> Result<(), Box<dyn Error>> {
    let (module, test_information) = hir_mir::compile_test(
        &compile_to_hir(infrastructure, source_file, dependency_file, &[])?,
        &prelude_type_configuration_qualifier::qualify(&compile_configuration.hir, PRELUDE_PREFIX),
        test_module_configuration,
    )?;

    compile_mir_module(
        infrastructure,
        &module,
        object_file,
        target_triple,
        compile_configuration,
    )?;

    infrastructure.file_system.write(
        test_information_file,
        &module_test_information_serializer::serialize(&test_information)?,
    )?;

    Ok(())
}

fn compile_to_hir(
    infrastructure: &Infrastructure,
    source_file: &FilePath,
    dependency_file: &FilePath,
    context_interfaces: &[interface::Module],
) -> Result<hir::ir::Module, Box<dyn Error>> {
    let (interface_files, prelude_interface_files) = dependency_serializer::deserialize(
        &infrastructure.file_system.read_to_vec(dependency_file)?,
    )?;

    let ast_module = parse::parse(
        &infrastructure.file_system.read_to_string(source_file)?,
        &infrastructure.file_path_displayer.display(source_file),
    )?;

    Ok(ast_hir::compile(
        &ast_module,
        &format!("{}:", source_file),
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
            .chain(context_interfaces.iter().cloned().map(Ok))
            .collect::<Result<Vec<_>, _>>()?,
    )?)
}

pub fn compile_prelude(
    infrastructure: &Infrastructure,
    source_file: &FilePath,
    object_file: &FilePath,
    interface_file: &FilePath,
    target_triple: Option<&str>,
    compile_configuration: &CompileConfiguration,
) -> Result<(), Box<dyn Error>> {
    let (module, module_interface) = hir_mir::compile_prelude(&ast_hir::compile_prelude(
        &parse::parse(
            &infrastructure.file_system.read_to_string(source_file)?,
            &infrastructure.file_path_displayer.display(source_file),
        )?,
        PRELUDE_PREFIX,
    )?)?;

    compile_mir_module(
        infrastructure,
        &module,
        object_file,
        target_triple,
        compile_configuration,
    )?;
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
    target_triple: Option<&str>,
    compile_configuration: &CompileConfiguration,
) -> Result<(), Box<dyn Error>> {
    infrastructure.file_system.write(
        object_file,
        &fmm_llvm::compile_to_bit_code(
            &fmm::analysis::cps::transform(
                &mir_fmm::compile(module, &compile_configuration.mir)?,
                fmm::types::void_type(),
            )?,
            &compile_configuration.fmm,
            target_triple,
        )?,
    )?;

    Ok(())
}
