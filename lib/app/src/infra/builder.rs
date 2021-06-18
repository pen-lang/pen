use super::file_path::FilePath;
use crate::build::ModuleBuildTarget;
use std::error::Error;

pub trait Builder {
    fn build(
        &self,
        package_prefix: &str,
        module_build_targets: &[ModuleBuildTarget],
        output_directory_path: &FilePath,
    ) -> Result<(), Box<dyn Error>>;
}
