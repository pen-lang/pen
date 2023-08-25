use super::{command_runner, file_path_converter::FilePathConverter};
use crate::{environment_variable_reader, InfrastructureError};
use std::{error::Error, path::PathBuf, process::Command, rc::Rc};

pub struct ExternalPackageInitializer {
    file_system: Rc<dyn app::infra::FileSystem>,
    file_path_converter: Rc<FilePathConverter>,
    language_root_scheme: &'static str,
    language_root_environment_variable: &'static str,
    packages_directory: &'static str,
}

impl ExternalPackageInitializer {
    pub fn new(
        file_system: Rc<dyn app::infra::FileSystem>,
        file_path_converter: Rc<FilePathConverter>,
        language_root_scheme: &'static str,
        language_root_environment_variable: &'static str,
        packages_directory: &'static str,
    ) -> Self {
        Self {
            file_system,
            file_path_converter,
            language_root_scheme,
            language_root_environment_variable,
            packages_directory,
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

        let directory = self
            .file_path_converter
            .convert_to_os_path(package_directory);

        if let Some(directory) = directory.parent() {
            std::fs::create_dir_all(directory)?;
        }

        match url.scheme() {
            "file" => {
                command_runner::run_command(
                    Command::new("cp").arg("-r").arg(url.path()).arg(directory),
                )?;
            }
            "git" => {
                command_runner::run_command(
                    Command::new("git")
                        .arg("clone")
                        .arg(url.as_str())
                        .arg(directory),
                )?;
            }
            _ => {
                if url.scheme() != self.language_root_scheme {
                    return Err(
                        InfrastructureError::PackageUrlSchemeNotSupported(url.clone()).into(),
                    );
                }

                command_runner::run_command(
                    Command::new("cp")
                        .arg("-r")
                        .arg(
                            PathBuf::from(environment_variable_reader::read(
                                self.language_root_environment_variable,
                            )?)
                            .join(self.packages_directory)
                            .join(url.path().strip_prefix('/').unwrap_or_default()),
                        )
                        .arg(directory),
                )?;
            }
        }

        Ok(())
    }
}
