use crate::{
    ApplicationConfiguration,
    common::file_path_resolver,
    error::ApplicationError,
    infra::{FilePath, Infrastructure},
    package_build_script_compiler,
};
use std::error::Error;

pub fn build(
    infrastructure: &Infrastructure,
    main_package_directory: &FilePath,
    output_directory: &FilePath,
    prelude_package_url: &url::Url,
    ffi_package_url: &url::Url,
    application_configuration: &ApplicationConfiguration,
) -> Result<(), Box<dyn Error>> {
    let build_script_file = package_build_script_compiler::compile(
        infrastructure,
        main_package_directory,
        output_directory,
        None,
        prelude_package_url,
        ffi_package_url,
        application_configuration,
    )?;

    infrastructure
        .build_script_runner
        .run(
            &build_script_file,
            &file_path_resolver::resolve_test_executable_file(output_directory),
        )
        .map_err(|_| ApplicationError::Build)?;

    Ok(())
}
