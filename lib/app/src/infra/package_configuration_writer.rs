use super::file_path::FilePath;
use std::{collections::HashMap, error::Error};

pub trait PackageConfigurationWriter {
    fn write(
        &self,
        dependencies: &HashMap<String, url::Url>,
        package_directory: &FilePath,
    ) -> Result<(), Box<dyn Error>>;
}
