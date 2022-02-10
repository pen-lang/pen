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
    let urls = external_package_configuration_reader::read_main(
        infrastructure,
        package_directory,
        output_directory,
    )?
    .into_iter()
    .filter_map(|(url, configuration)| {
        if configuration.is_system() {
            Some(url)
        } else {
            None
        }
    })
    .collect::<Vec<_>>();

    match urls.as_slice() {
        [] => Err(ApplicationError::SystemPackageNotFound.into()),
        [url] => Ok(url.clone()),
        _ => Err(ApplicationError::TooManySystemPackages.into()),
    }
}
