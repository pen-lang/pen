use super::command_runner;
use super::file_path_converter::FilePathConverter;
use crate::InfrastructureError;
use std::sync::Arc;
use std::{error::Error, process::Command};

pub struct ExternalPackageInitializer {
    file_path_converter: Arc<FilePathConverter>,
}

impl ExternalPackageInitializer {
    pub fn new(file_path_converter: Arc<FilePathConverter>) -> Self {
        Self {
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
        match url.scheme() {
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
