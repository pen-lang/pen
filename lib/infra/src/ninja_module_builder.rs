use super::{command_runner, file_path_converter::FilePathConverter};
use std::{error::Error, process::Stdio, sync::Arc};

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
            std::process::Command::new("sh")
                .arg("-ec")
                .arg(format!(
                    "(ninja -f {}; status=$?) | \
                        (grep -v ^ninja: || :) | \
                        sed s/FAILED/error/ | \
                        sed 's/^error:/\\x1b[1;31merror\\x1b[0m:/'; \
                        exit $status",
                    self.file_path_converter
                        .convert_to_os_path(build_script_file)
                        .display()
                ))
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit()),
        )?;

        Ok(())
    }
}
