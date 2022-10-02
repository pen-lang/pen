use std::{
    io,
    path::{Component, Path, PathBuf},
};

pub struct FilePathConverter {
    base_directory: PathBuf,
}

impl FilePathConverter {
    pub fn new(base_directory: impl AsRef<Path>) -> Self {
        Self {
            base_directory: base_directory.as_ref().into(),
        }
    }

    pub fn convert_to_os_path(&self, path: &app::infra::FilePath) -> PathBuf {
        self.base_directory.join(
            path.components()
                .map(|component| component.replace(['/', '\\'], "_"))
                .collect::<PathBuf>(),
        )
    }

    pub fn convert_to_file_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<app::infra::FilePath, Box<dyn std::error::Error>> {
        Ok(if path.as_ref().is_relative() {
            self.convert_components(path.as_ref())
        } else {
            self.convert_components(path.as_ref().strip_prefix(&self.base_directory).map_err(
                |_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!(
                            "path outside package directory: {}",
                            path.as_ref().to_string_lossy()
                        ),
                    )
                },
            )?)
        })
    }

    fn convert_components(&self, path: &Path) -> app::infra::FilePath {
        app::infra::FilePath::new(
            path.components()
                .filter_map(|component| match component {
                    Component::Normal(component) => Some(component.to_string_lossy().into()),
                    _ => None,
                })
                .collect::<Vec<String>>(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn convert_to_empty_os_path() {
        let base_directory = &env::current_dir().unwrap();

        assert_eq!(
            &FilePathConverter::new(base_directory)
                .convert_to_os_path(&app::infra::FilePath::new(Vec::<&str>::new())),
            base_directory
        );
    }

    #[test]
    fn convert_to_os_path() {
        let base_directory = &env::current_dir().unwrap();

        assert_eq!(
            FilePathConverter::new(base_directory)
                .convert_to_os_path(&app::infra::FilePath::new(vec!["foo"])),
            base_directory.join("foo")
        );
    }

    #[test]
    fn convert_to_os_path_escaping_path() {
        let base_directory = &env::current_dir().unwrap();

        assert_eq!(
            FilePathConverter::new(base_directory)
                .convert_to_os_path(&app::infra::FilePath::new(vec!["foo/bar\\baz"])),
            base_directory.join("foo_bar_baz")
        );
    }
}
