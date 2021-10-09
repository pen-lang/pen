use super::file_path::FilePath;
use std::{collections::BTreeMap, error::Error};

pub trait PackageConfigurationWriter {
    fn write(
        &self,
        dependencies: &BTreeMap<String, url::Url>,
        package_directory: &FilePath,
    ) -> Result<(), Box<dyn Error>>;
}
