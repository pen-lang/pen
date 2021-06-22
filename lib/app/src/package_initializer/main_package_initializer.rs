use super::package_initializer_infrastructure::PackageInitializerInfrastructure;
use crate::{
    common::calculate_package_id,
    infra::{FilePath, EXTERNAL_PACKAGE_DIRECTORY},
    package_build_script_compiler::{self, PackageBuildScriptCompilerInfrastructure},
};
use std::error::Error;

pub fn initialize(
    infrastructure: &PackageInitializerInfrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    initialize_external_packages(infrastructure, package_directory, output_directory)?;

    Ok(())
}

fn initialize_external_packages(
    infrastructure: &PackageInitializerInfrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    let package_configuration = infrastructure
        .package_configuration_reader
        .read(package_directory)?;

    for url in package_configuration.dependencies.values() {
        initialize_external_package(infrastructure, url, output_directory)?;
    }

    Ok(())
}

fn initialize_external_package(
    infrastructure: &PackageInitializerInfrastructure,
    url: &url::Url,
    output_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    let package_directory = output_directory.join(&FilePath::new(vec![
        EXTERNAL_PACKAGE_DIRECTORY.into(),
        calculate_package_id(url),
    ]));

    infrastructure
        .external_package_initializer
        .initialize(url, &package_directory)?;

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
    )?;

    initialize_external_packages(infrastructure, &package_directory, output_directory)?;

    Ok(())
}
