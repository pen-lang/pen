use super::{command_runner, file_path_converter::FilePathConverter};
use crate::{package_script_finder, InfrastructureError};
use std::{error::Error, process::Command, sync::Arc};

pub struct ApplicationLinker {
    file_path_converter: Arc<FilePathConverter>,
    link_script_basename: &'static str,
}

impl ApplicationLinker {
    pub fn new(
        file_path_converter: Arc<FilePathConverter>,
        link_script_basename: &'static str,
    ) -> Self {
        Self {
            file_path_converter,
            link_script_basename,
        }
    }
}

impl app::infra::ApplicationLinker for ApplicationLinker {
    fn link(
        &self,
        system_package_directory: &app::infra::FilePath,
        archive_files: &[app::infra::FilePath],
        application_file: &app::infra::FilePath,
        target: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let system_package_directory = self
            .file_path_converter
            .convert_to_os_path(system_package_directory);

        command_runner::run(
            Command::new(
                package_script_finder::find(&system_package_directory, self.link_script_basename)?
                    .ok_or(InfrastructureError::LinkScriptNotFound(
                        system_package_directory,
                    ))?,
            )
            .args(if let Some(target) = target {
                vec!["-t", target]
            } else {
                vec![]
            })
            .arg("-o")
            .arg(
                self.file_path_converter
                    .convert_to_os_path(application_file),
            )
            .args(
                archive_files
                    .iter()
                    .map(|file| self.file_path_converter.convert_to_os_path(file)),
            ),
        )?;

        Ok(())
    }
}
