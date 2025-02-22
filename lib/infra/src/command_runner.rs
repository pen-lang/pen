use super::error::InfrastructureError;
use crate::FilePathConverter;
use std::{
    error::Error,
    io::{Write, stderr, stdout},
    process::{Command, Stdio},
    rc::Rc,
};

pub struct CommandRunner {
    file_path_converter: Rc<FilePathConverter>,
}

impl CommandRunner {
    pub fn new(file_path_converter: Rc<FilePathConverter>) -> Self {
        Self {
            file_path_converter,
        }
    }
}

impl app::infra::CommandRunner for CommandRunner {
    fn run(&self, executable_file: &app::infra::FilePath) -> Result<(), Box<dyn Error>> {
        run_command(
            Command::new(self.file_path_converter.convert_to_os_path(executable_file))
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit()),
        )?;

        Ok(())
    }
}

pub fn run_command(command: &mut Command) -> Result<String, Box<dyn Error>> {
    let output = command.output()?;

    if output.status.success() {
        return Ok(String::from_utf8(output.stdout)?);
    }

    stdout().write_all(&output.stdout)?;
    stderr().write_all(&output.stderr)?;

    Err(InfrastructureError::CommandExit {
        status_code: output.status.code(),
    }
    .into())
}
