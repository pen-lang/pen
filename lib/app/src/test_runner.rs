use crate::{
    ApplicationConfiguration,
    common::file_path_resolver,
    error::ApplicationError,
    infra::{FilePath, Infrastructure},
    package_test_builder,
};
use std::error::Error;

pub fn run(
    infrastructure: &Infrastructure,
    main_package_directory: &FilePath,
    output_directory: &FilePath,
    prelude_package_url: &url::Url,
    ffi_package_url: &url::Url,
    application_configuration: &ApplicationConfiguration,
) -> Result<(), Box<dyn Error>> {
    package_test_builder::build(
        infrastructure,
        main_package_directory,
        output_directory,
        prelude_package_url,
        ffi_package_url,
        application_configuration,
    )?;

    infrastructure
        .command_runner
        .run(&file_path_resolver::resolve_test_executable_file(
            output_directory,
        ))
        .map_err(|_| ApplicationError::Test)?;

    Ok(())
}
