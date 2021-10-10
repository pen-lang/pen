use crate::{command_runner, environment_variable_reader, FilePathConverter};
use std::{error::Error, fs, path::Path, process::Command, sync::Arc};

pub struct TestLinker {
    file_path_converter: Arc<FilePathConverter>,
    language_root_environment_variable: &'static str,
}

impl TestLinker {
    pub fn new(
        file_path_converter: Arc<FilePathConverter>,
        language_root_environment_variable: &'static str,
    ) -> Self {
        Self {
            file_path_converter,
            language_root_environment_variable,
        }
    }
}

impl app::infra::TestLinker for TestLinker {
    fn link(
        &self,
        archive_files: &[app::infra::FilePath],
        test_information_file: &app::infra::FilePath,
        test_file: &app::infra::FilePath,
        test_directory: &app::infra::FilePath,
    ) -> Result<(), Box<dyn Error>> {
        let language_root_directory =
            environment_variable_reader::read(self.language_root_environment_variable)?;
        let main_crate_directory = self
            .file_path_converter
            .convert_to_os_path(test_directory)
            .join("main");

        if !main_crate_directory.exists() {
            command_runner::run_command(
                Command::new("cp")
                    .arg("-r")
                    .arg(&Path::new(&language_root_directory).join("cmd/test"))
                    .arg(&main_crate_directory),
            )?;
        }

        command_runner::run_command(
            Command::new("cargo")
                .arg("build")
                .arg("--release")
                .current_dir(&main_crate_directory)
                .envs([
                    (
                        "PEN_ARCHIVE_FILES",
                        archive_files
                            .iter()
                            .map(|file| {
                                self.file_path_converter
                                    .convert_to_os_path(file)
                                    .display()
                                    .to_string()
                            })
                            .collect::<Vec<_>>()
                            .join(":"),
                    ),
                    (
                        "PEN_TEST_INFORMATION_FILE",
                        self.file_path_converter
                            .convert_to_os_path(test_information_file)
                            .display()
                            .to_string(),
                    ),
                ]),
        )?;

        fs::copy(
            main_crate_directory.join("target/release/test"),
            self.file_path_converter.convert_to_os_path(test_file),
        )?;

        Ok(())
    }
}
