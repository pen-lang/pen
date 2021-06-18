use super::FilePath;
use std::error::Error;

pub trait Builder {
    fn build(&self, file_paths: &[FilePath]) -> Result<(), Box<dyn Error>>;
}
