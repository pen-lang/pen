use crate::{
    error::ApplicationError,
    external_package_configuration_reader,
    infra::{FilePath, Infrastructure},
};
use fnv::FnvHashMap;
use std::error::Error;

// TODO Use a package type field.
pub fn find(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
) -> Result<FnvHashMap<String, url::Url>, Box<dyn Error>> {
    let package_configuration = infrastructure
        .package_configuration_reader
        .read(package_directory)?;
    let system_packages = external_package_configuration_reader::read(
        infrastructure,
        &package_configuration,
        output_directory,
    )?
    .into_iter()
    .filter_map(|(key, configuration)| {
        if configuration.is_system() {
            Some((
                key.clone(),
                package_configuration.dependencies()[&key].clone(),
            ))
        } else {
            None
        }
    })
    .collect::<FnvHashMap<_, _>>();

    if system_packages.is_empty() {
        Err(ApplicationError::SystemPackageNotFound.into())
    } else {
        Ok(system_packages)
    }
}
