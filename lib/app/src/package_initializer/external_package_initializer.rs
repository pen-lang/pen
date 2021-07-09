use crate::{
    common::file_path_resolver,
    infra::{FilePath, Infrastructure},
    package_build_script_compiler,
};
use std::error::Error;

pub fn initialize_recursively(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    let package_configuration = infrastructure
        .package_configuration_reader
        .read(package_directory)?;

    for url in package_configuration.dependencies.values() {
        initialize(infrastructure, url, output_directory)?;
    }

    Ok(())
}

fn initialize(
    infrastructure: &Infrastructure,
    package_url: &url::Url,
    output_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    let package_directory =
        file_path_resolver::resolve_package_directory(output_directory, package_url);

    infrastructure
        .external_package_initializer
        .initialize(package_url, &package_directory)?;

    package_build_script_compiler::compile_external(
        infrastructure,
        package_url,
        output_directory,
        &package_directory.with_extension(
            infrastructure
                .file_path_configuration
                .build_script_file_extension,
        ),
    )?;

    initialize_recursively(infrastructure, &package_directory, output_directory)?;

    Ok(())
}
