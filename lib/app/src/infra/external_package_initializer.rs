use super::file_path::FilePath;
use std::error::Error;

pub trait ExternalPackageInitializer {
    fn initialize(
        &self,
        url: &url::Url,
        package_directory: &FilePath,
    ) -> Result<(), Box<dyn Error>>;
}
