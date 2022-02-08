use super::file_path::FilePath;
use crate::package_configuration::PackageConfiguration;
use std::error::Error;

pub trait PackageConfigurationReader {
    fn read(&self, package_directory: &FilePath) -> Result<PackageConfiguration, Box<dyn Error>>;
}
