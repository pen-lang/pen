use super::file_path::FilePath;
use crate::package_builder::ModuleTarget;
use std::error::Error;

pub trait ModuleBuilder {
    fn build(
        &self,
        module_targets: &[ModuleTarget],
        output_directory: &FilePath,
    ) -> Result<(), Box<dyn Error>>;
}
