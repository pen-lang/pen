use super::command_runner;
use super::file_path_converter::FilePathConverter;
use crate::InfrastructureError;
use std::sync::Arc;
use std::{error::Error, process::Command};

pub struct ExternalPackageInitializer {
    file_system: Arc<dyn app::infra::FileSystem>,
    file_path_converter: Arc<FilePathConverter>,
}

impl ExternalPackageInitializer {
    pub fn new(
        file_system: Arc<dyn app::infra::FileSystem>,
        file_path_converter: Arc<FilePathConverter>,
    ) -> Self {
        Self {
            file_system,
            file_path_converter,
        }
    }
}

impl app::infra::ExternalPackageInitializer for ExternalPackageInitializer {
    fn initialize(
        &self,
        url: &url::Url,
        package_directory: &app::infra::FilePath,
    ) -> Result<(), Box<dyn Error>> {
        if self.file_system.exists(package_directory) {
            return Ok(());
        }

        match url.scheme() {
            "file" => {
                command_runner::run(
                    Command::new("cp").arg("-r").arg(url.path()).arg(
                        self.file_path_converter
                            .convert_to_os_path(package_directory),
                    ),
                )?;
            }
            "git" => {
                command_runner::run(
                    Command::new("git").arg("clone").arg(url.as_str()).arg(
                        self.file_path_converter
                            .convert_to_os_path(package_directory),
                    ),
                )?;
            }
            _ => return Err(InfrastructureError::PackageUrlSchemeNotSupported(url.clone()).into()),
        }

        Ok(())
    }
}
