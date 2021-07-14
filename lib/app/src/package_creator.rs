use crate::{
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
        &package_directory.join(&FilePath::new([
            &application_configuration.main_module_basename
        ])),
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
        &package_directory.join(&FilePath::new([module_filename])),
        module_content.as_bytes(),
    )?;

    Ok(())
}
