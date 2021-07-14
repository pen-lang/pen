
use crate::infra::PackageConfiguration;
use crate::{
    infra::{FilePath, Infrastructure},
};
use std::error::Error;

pub fn create(
    infrastructure: &Infrastructure,
    module_file: &FilePath,
    module_content: &str,
    package_configuration: &PackageConfiguration,
    package_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    infrastructure
        .package_configuration_writer
        .write(package_configuration, package_directory)?;

    infrastructure
        .file_system
        .write(module_file, module_content.as_bytes())?;

    Ok(())
}
