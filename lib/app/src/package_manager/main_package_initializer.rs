use super::package_manager_infrastructure::PackageManagerInfrastructure;
use crate::{
    common::calculate_package_id,
    infra::{FilePath, PACKAGE_DIRECTORY},
};
use std::error::Error;

pub fn initialize_main_package(
    infrastructure: &PackageManagerInfrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    initialize_external_packages(
        infrastructure,
        package_directory,
        &output_directory.join(&FilePath::new(vec![PACKAGE_DIRECTORY])),
    )?;

    Ok(())
}

fn initialize_external_packages(
    infrastructure: &PackageManagerInfrastructure,
    package_directory: &FilePath,
    external_package_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    let package_configuration = infrastructure
        .package_configuration_reader
        .read(package_directory)?;

    for url in package_configuration.dependencies.values() {
        initialize_external_package(infrastructure, url, external_package_directory)?;
    }

    Ok(())
}

fn initialize_external_package(
    infrastructure: &PackageManagerInfrastructure,
    url: &url::Url,
    external_package_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    let package_directory =
        external_package_directory.join(&FilePath::new(vec![calculate_package_id(url)]));

    infrastructure
        .external_package_initializer
        .initialize(url, &package_directory)?;

    initialize_external_packages(
        infrastructure,
        &package_directory,
        external_package_directory,
    )?;

    Ok(())
}
