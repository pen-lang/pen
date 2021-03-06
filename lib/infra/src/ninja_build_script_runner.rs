use super::file_path_converter::FilePathConverter;
use crate::command_runner;
use std::{
    error::Error,
    process::{Command, Stdio},
    sync::Arc,
};

pub struct NinjaBuildScriptRunner {
    file_path_converter: Arc<FilePathConverter>,
}

impl NinjaBuildScriptRunner {
    pub fn new(file_path_converter: Arc<FilePathConverter>) -> Self {
        Self {
            file_path_converter,
        }
    }
}

impl app::infra::BuildScriptRunner for NinjaBuildScriptRunner {
    fn run(&self, build_script_file: &app::infra::FilePath) -> Result<(), Box<dyn Error>> {
        command_runner::run_command(
            Command::new("turtle")
                .arg("--quiet")
                .arg("-f")
                .arg(
                    self.file_path_converter
                        .convert_to_os_path(build_script_file),
                )
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit()),
        )?;

        Ok(())
    }
}
