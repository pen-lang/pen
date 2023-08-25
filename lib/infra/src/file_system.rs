use super::{error::InfrastructureError, file_path_converter::FilePathConverter};
use std::{io::Error, rc::Rc};

pub struct FileSystem {
    file_path_converter: Rc<FilePathConverter>,
}

impl FileSystem {
    pub fn new(file_path_converter: Rc<FilePathConverter>) -> Self {
        Self {
            file_path_converter,
        }
    }

    fn read_directory_with_raw_error(
        &self,
        file_path: &app::infra::FilePath,
    ) -> Result<Vec<app::infra::FilePath>, Error> {
        let path = self.file_path_converter.convert_to_os_path(file_path);

        path.read_dir()?
            .map(|entry| {
                Ok(self
                    .file_path_converter
                    .convert_to_file_path(entry?.path())
                    .unwrap())
            })
            .collect::<Result<_, Error>>()
    }
}

impl app::infra::FileSystem for FileSystem {
    fn exists(&self, file_path: &app::infra::FilePath) -> bool {
        self.file_path_converter
            .convert_to_os_path(file_path)
            .exists()
    }

    fn is_directory(&self, file_path: &app::infra::FilePath) -> bool {
        self.file_path_converter
            .convert_to_os_path(file_path)
            .is_dir()
    }

    fn read_directory(
        &self,
        file_path: &app::infra::FilePath,
    ) -> Result<Vec<app::infra::FilePath>, Box<dyn std::error::Error>> {
        Ok(self
            .read_directory_with_raw_error(file_path)
            .map_err(|source| InfrastructureError::ReadDirectory {
                path: self.file_path_converter.convert_to_os_path(file_path),
                source,
            })?)
    }

    fn read_to_vec(
        &self,
        file_path: &app::infra::FilePath,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let path = self.file_path_converter.convert_to_os_path(file_path);

        Ok(
            std::fs::read(&path)
                .map_err(|source| InfrastructureError::ReadFile { path, source })?,
        )
    }

    fn read_to_string(
        &self,
        file_path: &app::infra::FilePath,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let path = self.file_path_converter.convert_to_os_path(file_path);

        Ok(std::fs::read_to_string(&path)
            .map_err(|source| InfrastructureError::ReadFile { path, source })?)
    }

    fn write(
        &self,
        file_path: &app::infra::FilePath,
        data: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.file_path_converter.convert_to_os_path(file_path);

        if let Some(directory) = path.parent() {
            std::fs::create_dir_all(directory).map_err(|source| {
                InfrastructureError::CreateDirectory {
                    path: directory.into(),
                    source,
                }
            })?;
        }

        std::fs::write(&path, data)
            .map_err(|source| InfrastructureError::WriteFile { path, source })?;

        Ok(())
    }
}
