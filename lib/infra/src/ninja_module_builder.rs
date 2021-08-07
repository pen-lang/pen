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
        // spell-checker:disable
        command_runner::run(
            std::process::Command::new("bash")
                .arg("-o")
                .arg("pipefail")
                .arg("-ec")
                .arg(format!(
                    "ninja -f {} | \
                        (stdbuf -o0 grep -v ^ninja: || :) | \
                        stdbuf -o0 sed s/FAILED/error/ | \
                        stdbuf -o0 sed 's/^error:/\\x1b[0;31merror\\x1b[0m:/'",
                    self.file_path_converter
                        .convert_to_os_path(build_script_file)
                        .display()
                ))
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit()),
        )?;
        // spell-checker:enable

        Ok(())
    }
}
