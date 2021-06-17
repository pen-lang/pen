use super::file_path_converter::FilePathConverter;
use std::sync::Arc;

pub struct FilePathDisplayer {
    file_path_converter: Arc<FilePathConverter>,
}

impl FilePathDisplayer {
    pub fn new(file_path_converter: Arc<FilePathConverter>) -> Self {
        Self {
            file_path_converter,
        }
    }
}

impl app::infra::FilePathDisplayer for FilePathDisplayer {
    fn display(&self, file_path: &app::infra::FilePath) -> String {
        format!(
            "{}",
            self.file_path_converter
                .convert_to_os_path(file_path)
                .canonicalize()
                .expect("valid os file path")
                .display()
        )
    }
}
