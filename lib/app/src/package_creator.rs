use crate::{
    common::file_path_resolver,
    infra::{FilePath, Infrastructure, PackageConfiguration},
    ApplicationConfiguration,
};
use std::{collections::HashMap, error::Error};

pub fn create_application(
    infrastructure: &Infrastructure,
    module_content: &str,
    system_package_url: &url::Url,
    application_configuration: &ApplicationConfiguration,
    package_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    create(
        infrastructure,
        &vec![(
            application_configuration.system_package_name.clone(),
            system_package_url.clone(),
        )]
        .into_iter()
        .collect(),
        &application_configuration.main_module_basename,
        module_content,
        package_directory,
    )
}

pub fn create_library(
    infrastructure: &Infrastructure,
    module_basename: &str,
    module_content: &str,
    package_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    create(
        infrastructure,
        &Default::default(),
        module_basename,
        module_content,
        package_directory,
    )
}

fn create(
    infrastructure: &Infrastructure,
    dependencies: &HashMap<String, url::Url>,
    module_basename: &str,
    module_content: &str,
    package_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    infrastructure.package_configuration_writer.write(
        &PackageConfiguration {
            dependencies: dependencies.clone(),
        },
        package_directory,
    )?;

    infrastructure.file_system.write(
        &file_path_resolver::resolve_source_file(
            package_directory,
            &[module_basename.into()],
            &infrastructure.file_path_configuration,
        ),
        module_content.as_bytes(),
    )?;

    Ok(())
}
