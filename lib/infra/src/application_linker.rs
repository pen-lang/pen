use super::{command_runner, file_path_converter::FilePathConverter};
use std::{error::Error, sync::Arc};

pub struct ApplicationLinker {
    file_path_converter: Arc<FilePathConverter>,
}

impl ApplicationLinker {
    pub fn new(file_path_converter: Arc<FilePathConverter>) -> Self {
        Self {
            file_path_converter,
        }
    }
}

impl app::infra::ApplicationLinker for ApplicationLinker {
    fn link(
        &self,
        object_files: &[app::infra::FilePath],
        archive_files: &[app::infra::FilePath],
        application_file: &app::infra::FilePath,
        target: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        command_runner::run(
            std::process::Command::new("clang")
                .arg("-Werror")
                .arg("-static")
                .args(if let Some(target) = target {
                    vec!["-target", target]
                } else {
                    vec![]
                })
                .arg("-o")
                .arg(
                    self.file_path_converter
                        .convert_to_os_path(application_file),
                )
                .args(
                    object_files
                        .iter()
                        .map(|file| self.file_path_converter.convert_to_os_path(file)),
                )
                .args(
                    archive_files
                        .iter()
                        .map(|file| self.file_path_converter.convert_to_os_path(file)),
                )
                .arg("-ldl")
                .arg("-lpthread"),
        )?;

        Ok(())
    }
}
