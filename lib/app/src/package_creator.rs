use crate::{
    ApplicationConfiguration,
    common::file_path_resolver,
    infra::{FilePath, Infrastructure},
    package_configuration::{PackageConfiguration, PackageType},
};
use std::{collections::BTreeMap, error::Error};

pub fn create_application(
    infrastructure: &Infrastructure,
    module_content: &str,
    system_package_name: &str,
    system_package_url: &url::Url,
    application_configuration: &ApplicationConfiguration,
    package_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    create(
        infrastructure,
        PackageType::Application,
        &[(system_package_name.into(), system_package_url.clone())]
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
        PackageType::Library,
        &Default::default(),
        module_basename,
        module_content,
        package_directory,
    )
}

fn create(
    infrastructure: &Infrastructure,
    package_type: PackageType,
    dependencies: &BTreeMap<String, url::Url>,
    module_basename: &str,
    module_content: &str,
    package_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    infrastructure.package_configuration_writer.write(
        &PackageConfiguration::new(package_type, dependencies.clone()),
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
