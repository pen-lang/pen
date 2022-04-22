use super::FilePath;
use std::error::Error;

pub trait TestLinker {
    fn link(
        &self,
        package_test_information: &test::Package,
        archive_files: &[FilePath],
        test_file: &FilePath,
        output_directory: &FilePath,
    ) -> Result<(), Box<dyn Error>>;
}
