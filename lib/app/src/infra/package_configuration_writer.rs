use super::{file_path::FilePath, package_configuration::PackageConfiguration};
use std::error::Error;

pub trait PackageConfigurationWriter {
    fn write(
        &self,
        package_configuration: &PackageConfiguration,
        package_directory: &FilePath,
    ) -> Result<(), Box<dyn Error>>;
}
