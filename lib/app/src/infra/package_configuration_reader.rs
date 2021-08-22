use super::file_path::FilePath;
use std::{collections::HashMap, error::Error};

pub trait PackageConfigurationReader {
    fn get_dependencies(
        &self,
        package_directory: &FilePath,
    ) -> Result<HashMap<String, url::Url>, Box<dyn Error>>;

    fn is_ffi_enabled(&self, package_directory: &FilePath) -> Result<bool, Box<dyn Error>>;
}
