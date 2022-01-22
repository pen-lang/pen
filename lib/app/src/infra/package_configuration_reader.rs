use super::file_path::FilePath;
use std::{collections::BTreeMap, error::Error};

pub trait PackageConfigurationReader {
    fn get_dependencies(
        &self,
        package_directory: &FilePath,
    ) -> Result<BTreeMap<String, url::Url>, Box<dyn Error>>;
}
