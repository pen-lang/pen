use super::application_configuration::ApplicationConfiguration;
use crate::{
    PackageType,
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
        .run(
            &build_script_file,
            &if infrastructure
                .package_configuration_reader
                .read(main_package_directory)?
                .type_()
                == PackageType::Application
            {
                file_path_resolver::resolve_application_file(
                    main_package_directory,
                    application_configuration,
                )
            } else {
                file_path_resolver::resolve_main_package_archive_file(
                    output_directory,
                    &infrastructure.file_path_configuration,
                )
            },
        )
        .map_err(|_| ApplicationError::Build)?;

    Ok(())
}
