use crate::{command_runner, environment_variable_reader, FilePathConverter};
use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    process::Command,
    sync::Arc,
};

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

    fn format_main_rs(
        &self,
        package_test_information: &test_info::Package,
    ) -> Result<String, Box<dyn Error>> {
        Ok(format!(
            r#"
            mod debug;
            mod heap;
            mod spawn;
            mod test_result;
            mod unreachable;

            use test_result::TestResult;

            extern "C" {{
                fn _pen_test_convert_result(result: ffi::Any) -> ffi::Arc<TestResult>;
            }}

            fn main() {{
                #[allow(unused_mut)]
                let mut success: usize = 0;
                #[allow(unused_mut)]
                let mut error: usize = 0;

                {}

                println!("test summary");
                println!(
                    "\t{{}}\t{{}} passed, {{}} failed",
                    if error == 0 {{ "OK" }} else {{ "FAIL" }},
                    success, error
                );

                if error > 0 {{
                    std::process::exit(1);
                }}
            }}
            "#,
            self.format_tests(package_test_information)?,
        ))
    }

    fn format_tests(
        &self,
        package_test_information: &test_info::Package,
    ) -> Result<String, Box<dyn Error>> {
        Ok(package_test_information
            .modules()
            .iter()
            .map(|(name, module)| {
                format!(r#"println!("{}");"#, name)
                    + &module
                        .functions()
                        .iter()
                        .map(|function| {
                            self.format_test_function(function.name(), function.foreign_name())
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
            })
            .collect::<Vec<_>>()
            .join("\n"))
    }

    fn format_test_function(&self, name: &str, foreign_name: &str) -> String {
        format!(
            r#"
            #[link(name = "main_test")]
            extern "C" {{ fn {foreign_name}() -> ffi::Any; }}

            let result: Result<_, _>
                = unsafe {{ _pen_test_convert_result({foreign_name}()) }}.to_result();
            println!("\t{{}}\t{name}", if result.is_ok() {{ "OK" }} else {{ "FAIL" }});

            if let Err(message) = &result {{
                println!("\t\tMessage: {{}}", message);
                error += 1;
            }} else {{
                success += 1;
            }}
            "#,
            name = name,
            foreign_name = foreign_name,
        )
    }
}

impl app::infra::TestLinker for TestLinker {
    fn link(
        &self,
        package_test_information: &test_info::Package,
        archive_files: &[app::infra::FilePath],
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

        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&main_crate_directory.join("src/main.rs"))?
            .write_all(self.format_main_rs(package_test_information)?.as_bytes())?;

        command_runner::run_command(
            Command::new("cargo")
                .arg("build")
                .arg("--release")
                .current_dir(&main_crate_directory)
                .envs([(
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
                )]),
        )?;

        fs::copy(
            main_crate_directory.join("target/release/test"),
            self.file_path_converter.convert_to_os_path(test_file),
        )?;

        Ok(())
    }
}
