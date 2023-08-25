use super::file_path_converter::FilePathConverter;
use crate::command_runner;
use std::{
    error::Error,
    process::{Command, Stdio},
    rc::Rc,
};

pub struct NinjaBuildScriptRunner {
    file_path_converter: Rc<FilePathConverter>,
}

impl NinjaBuildScriptRunner {
    pub fn new(file_path_converter: Rc<FilePathConverter>) -> Self {
        Self {
            file_path_converter,
        }
    }
}

impl app::infra::BuildScriptRunner for NinjaBuildScriptRunner {
    fn run(
        &self,
        build_script_file: &app::infra::FilePath,
        target_file: &app::infra::FilePath,
    ) -> Result<(), Box<dyn Error>> {
        let build_script_file = self
            .file_path_converter
            .convert_to_os_path(build_script_file);

        command_runner::run_command(
            Command::new("turtle")
                .arg("-f")
                .arg(&build_script_file)
                .arg("-t")
                .arg("cleandead")
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit()),
        )?;

        command_runner::run_command(
            Command::new("turtle")
                .arg("--quiet")
                .arg("-f")
                .arg(&build_script_file)
                .arg(&self.file_path_converter.convert_to_os_path(target_file))
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit()),
        )?;

        Ok(())
    }
}
