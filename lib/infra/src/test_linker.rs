use crate::{FilePathConverter, command_runner, environment_variable_reader};
use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    process::Command,
    rc::Rc,
};

pub struct TestLinker {
    file_path_converter: Rc<FilePathConverter>,
    language_root_environment_variable: &'static str,
}

impl TestLinker {
    pub fn new(
        file_path_converter: Rc<FilePathConverter>,
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
            mod unreachable;

            fn main() {{
                #[allow(unused_mut)]
                let mut success: usize = 0;
                #[allow(unused_mut)]
                let mut error: usize = 0;

                {}

                println!("summary");
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
                format!(r#"println!("{name}");"#)
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
            unsafe extern "C" {{ fn {foreign_name}() -> ffi::ByteString; }}

            let message = unsafe {{ {foreign_name}() }};
            println!("\t{{}}\t{name}", if message.as_slice().is_empty() {{ "OK" }} else {{ "FAIL" }});

            if message.as_slice().is_empty() {{
                success += 1;
            }} else {{
                println!("\t\tMessage: {{}}", String::from_utf8_lossy(message.as_slice()));
                error += 1;
            }}
            "#,
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
                    .arg(Path::new(&language_root_directory).join("cmd/test"))
                    .arg(&main_crate_directory),
            )?;
        }

        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(main_crate_directory.join("src/main.rs"))?
            .write_all(self.format_main_rs(package_test_information)?.as_bytes())?;

        command_runner::run_command(
            Command::new("cargo")
                .arg("build")
                .arg("--release")
                .current_dir(&main_crate_directory)
                .envs([(
                    "PEN_TEST_ARCHIVES",
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
