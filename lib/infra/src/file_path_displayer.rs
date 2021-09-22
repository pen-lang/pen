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
        let os_path = self.file_path_converter.convert_to_os_path(file_path);

        format!("{}", os_path.canonicalize().unwrap_or(os_path).display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use app::infra::FilePathDisplayer;

    #[test]
    fn display_path() {
        assert_eq!(
            super::FilePathDisplayer::new(FilePathConverter::new("/").into())
                .display(&app::infra::FilePath::new(vec!["foo", "bar"])),
            "/foo/bar"
        );
    }
}
