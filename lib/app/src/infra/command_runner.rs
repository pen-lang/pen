use super::file_path::FilePath;
use std::error::Error;

pub trait CommandRunner {
    fn run(&self, executable_file: &FilePath) -> Result<(), Box<dyn Error>>;
}
