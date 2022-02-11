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
    let system_packages = external_package_configuration_reader::read_main(
        infrastructure,
        package_directory,
        output_directory,
    )?
    .into_iter()
    .filter_map(|(key, (url, configuration))| {
        if configuration.is_system() {
            Some((key, url))
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
