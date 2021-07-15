use crate::{
    common::file_path_resolver,
    infra::{FilePath, Infrastructure, PackageConfiguration},
    ApplicationConfiguration,
};
use std::error::Error;

pub fn create_application(
    infrastructure: &Infrastructure,
    module_content: &str,
    system_package_url: &url::Url,
    application_configuration: &ApplicationConfiguration,
    package_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    infrastructure.package_configuration_writer.write(
        &PackageConfiguration {
            dependencies: vec![(
                application_configuration.system_package_name.clone(),
                system_package_url.clone(),
            )]
            .into_iter()
            .collect(),
        },
        package_directory,
    )?;

    infrastructure.file_system.write(
        &file_path_resolver::resolve_source_file(
            package_directory,
            &[application_configuration.main_module_basename.clone()],
            &infrastructure.file_path_configuration,
        ),
        module_content.as_bytes(),
    )?;

    Ok(())
}

pub fn create_library(
    infrastructure: &Infrastructure,
    module_filename: &str,
    module_content: &str,
    package_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    infrastructure.package_configuration_writer.write(
        &PackageConfiguration {
            dependencies: Default::default(),
        },
        package_directory,
    )?;

    infrastructure.file_system.write(
        &file_path_resolver::resolve_source_file(
            package_directory,
            &[module_filename.into()],
            &infrastructure.file_path_configuration,
        ),
        module_content.as_bytes(),
    )?;

    Ok(())
}
