use super::{command_runner, file_path_converter::FilePathConverter};
use std::{error::Error, sync::Arc};

pub struct NinjaModuleBuilder {
    file_path_converter: Arc<FilePathConverter>,
}

impl NinjaModuleBuilder {
    pub fn new(file_path_converter: Arc<FilePathConverter>) -> Self {
        Self {
            file_path_converter,
        }
    }
}

impl app::infra::ModuleBuilder for NinjaModuleBuilder {
    fn build(&self, build_script_file: &app::infra::FilePath) -> Result<(), Box<dyn Error>> {
        command_runner::run(
            std::process::Command::new("ninja").arg("-f").arg(
                self.file_path_converter
                    .convert_to_os_path(build_script_file),
            ),
        )?;

        Ok(())
    }
}
