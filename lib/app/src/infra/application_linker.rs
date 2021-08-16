use super::file_path::FilePath;
use std::error::Error;

pub trait ApplicationLinker {
    fn link(
        &self,
        system_package_directory: &FilePath,
        archive_files: &[FilePath],
        application_file: &FilePath,
        target_triple: Option<&str>,
    ) -> Result<(), Box<dyn Error>>;
}
