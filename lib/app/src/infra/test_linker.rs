use super::FilePath;
use std::error::Error;

pub trait TestLinker {
    fn link(
        &self,
        archive_files: &[FilePath],
        test_information_file: &FilePath,
        test_file: &FilePath,
        output_directory: &FilePath,
    ) -> Result<(), Box<dyn Error>>;
}
