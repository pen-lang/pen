use super::file_path::FilePath;
use std::error::Error;

pub trait ModuleBuilder {
    fn build(&self, build_script_file: &FilePath) -> Result<(), Box<dyn Error>>;
}
