use super::file_path::FilePath;
use crate::package_configuration::PackageConfiguration;
use std::error::Error;

pub trait PackageConfigurationWriter {
    fn write(
        &self,
        configuration: &PackageConfiguration,
        package_directory: &FilePath,
    ) -> Result<(), Box<dyn Error>>;
}
