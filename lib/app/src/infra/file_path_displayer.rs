use super::FilePath;

pub trait FilePathDisplayer {
    fn display(&self, file_path: &FilePath) -> String;
}
