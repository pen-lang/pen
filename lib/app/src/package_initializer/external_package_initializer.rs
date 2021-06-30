use super::package_initializer_infrastructure::PackageInitializerInfrastructure;
use crate::{
    common::file_path_resolver,
    infra::FilePath,
    package_build_script_compiler::{self, PackageBuildScriptCompilerInfrastructure},
};
use std::error::Error;

pub fn initialize_recursively(
    infrastructure: &PackageInitializerInfrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
    prelude_package_url: &url::Url,
) -> Result<(), Box<dyn Error>> {
    let package_configuration = infrastructure
        .package_configuration_reader
        .read(package_directory)?;

    for url in package_configuration.dependencies.values() {
        initialize(infrastructure, url, output_directory, prelude_package_url)?;
    }

    Ok(())
}

fn initialize(
    infrastructure: &PackageInitializerInfrastructure,
    package_url: &url::Url,
    output_directory: &FilePath,
    prelude_package_url: &url::Url,
) -> Result<(), Box<dyn Error>> {
    let package_directory =
        file_path_resolver::resolve_package_directory(output_directory, package_url);

    infrastructure
        .external_package_initializer
        .initialize(package_url, &package_directory)?;

    package_build_script_compiler::compile(
        &PackageBuildScriptCompilerInfrastructure {
            module_build_script_compiler: infrastructure.module_build_script_compiler.clone(),
            file_system: infrastructure.file_system.clone(),
            file_path_configuration: infrastructure.file_path_configuration.clone(),
        },
        &package_directory,
        output_directory,
        &[],
        &package_directory.with_extension(
            infrastructure
                .file_path_configuration
                .build_script_file_extension,
        ),
        prelude_package_url,
    )?;

    initialize_recursively(
        infrastructure,
        &package_directory,
        output_directory,
        prelude_package_url,
    )?;

    Ok(())
}
