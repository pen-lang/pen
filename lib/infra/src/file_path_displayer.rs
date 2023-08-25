use super::file_path_converter::FilePathConverter;
use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

pub struct FilePathDisplayer {
    file_path_converter: Rc<FilePathConverter>,
    package_directory: PathBuf,
}

impl FilePathDisplayer {
    pub fn new(
        file_path_converter: Rc<FilePathConverter>,
        package_directory: impl AsRef<Path>,
    ) -> Self {
        Self {
            file_path_converter,
            package_directory: package_directory.as_ref().into(),
        }
    }
}

impl app::infra::FilePathDisplayer for FilePathDisplayer {
    fn display(&self, file_path: &app::infra::FilePath) -> String {
        let os_path = self.file_path_converter.convert_to_os_path(file_path);

        format!(
            "{}",
            os_path
                .canonicalize()
                .as_ref()
                .unwrap_or(&os_path)
                .strip_prefix(&self.package_directory)
                .unwrap_or(&os_path)
                .display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use app::infra::FilePathDisplayer;

    #[test]
    fn display_path() {
        assert_eq!(
            super::FilePathDisplayer::new(FilePathConverter::new("/").into(), "/")
                .display(&app::infra::FilePath::new(vec!["foo", "bar"])),
            "foo/bar"
        );
    }

    #[test]
    fn display_relative_path() {
        assert_eq!(
            super::FilePathDisplayer::new(FilePathConverter::new("/foo").into(), "/foo")
                .display(&app::infra::FilePath::new(vec!["bar", "baz"])),
            "bar/baz"
        );
    }
}
