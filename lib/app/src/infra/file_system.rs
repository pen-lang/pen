use super::FilePath;
use std::error::Error;

pub trait FileSystem {
    fn exists(&self, path: &FilePath) -> bool;
    fn is_directory(&self, path: &FilePath) -> bool;
    fn read_directory(&self, path: &FilePath) -> Result<Vec<FilePath>, Box<dyn Error>>;
    fn read_to_string(&self, path: &FilePath) -> Result<String, Box<dyn Error>>;
    fn read_to_vec(&self, path: &FilePath) -> Result<Vec<u8>, Box<dyn Error>>;
    fn write(&self, path: &FilePath, data: &[u8]) -> Result<(), Box<dyn Error>>;
}
