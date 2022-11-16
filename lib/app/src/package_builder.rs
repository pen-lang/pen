use super::application_configuration::ApplicationConfiguration;
use crate::{
    error::ApplicationError,
    infra::{FilePath, Infrastructure},
    package_build_script_compiler,
};
use std::error::Error;

pub fn build(
    infrastructure: &Infrastructure,
    main_package_directory: &FilePath,
    output_directory: &FilePath,
    target_triple: Option<&str>,
    prelude_package_url: &url::Url,
    ffi_package_url: &url::Url,
    application_configuration: &ApplicationConfiguration,
) -> Result<(), Box<dyn Error>> {
    let build_script_file = package_build_script_compiler::compile(
        infrastructure,
        main_package_directory,
        output_directory,
        target_triple,
        prelude_package_url,
        ffi_package_url,
        application_configuration,
    )?;

    infrastructure
        .build_script_runner
        .run(&build_script_file)
        .map_err(|_| ApplicationError::Build)?;

    Ok(())
}
