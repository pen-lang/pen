use super::file_path::FilePath;
use std::error::Error;

pub trait BuildScriptRunner {
    fn run(
        &self,
        build_script_file: &FilePath,
        target_file: &FilePath,
    ) -> Result<(), Box<dyn Error>>;
}
