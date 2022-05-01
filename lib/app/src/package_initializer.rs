mod external_package_initializer;

use crate::{
    common::file_path_resolver,
    infra::{FilePath, Infrastructure},
    package_build_script_compiler,
};
use std::error::Error;

pub fn initialize(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
    prelude_package_url: &url::Url,
    ffi_package_url: &url::Url,
) -> Result<(), Box<dyn Error>> {
    initialize_prelude(infrastructure, prelude_package_url, output_directory)?;
    external_package_initializer::initialize(infrastructure, ffi_package_url, output_directory)?;

    external_package_initializer::initialize_dependencies(
        infrastructure,
        package_directory,
        output_directory,
    )?;

    Ok(())
}

fn initialize_prelude(
    infrastructure: &Infrastructure,
    package_url: &url::Url,
    output_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    let package_directory =
        file_path_resolver::resolve_package_directory(output_directory, package_url);

    infrastructure
        .external_package_initializer
        .initialize(package_url, &package_directory)?;

    package_build_script_compiler::compile_prelude(
        infrastructure,
        package_url,
        output_directory,
        &file_path_resolver::resolve_external_package_build_script_file(
            output_directory,
            package_url,
            &infrastructure.file_path_configuration,
        ),
    )?;

    Ok(())
}
