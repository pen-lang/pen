use super::command_runner;
use super::file_path_converter::FilePathConverter;
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

pub struct Builder {
    file_system: Arc<dyn app::infra::FileSystem>,
    file_path_converter: Arc<FilePathConverter>,
    file_path_configuration: Arc<app::common::FilePathConfiguration>,
    ninja_directory: &'static str,
}

impl Builder {
    pub fn new(
        file_system: Arc<dyn app::infra::FileSystem>,
        file_path_converter: Arc<FilePathConverter>,
        file_path_configuration: Arc<app::common::FilePathConfiguration>,
        ninja_directory: &'static str,
    ) -> Self {
        Self {
            file_system,
            file_path_converter,
            file_path_configuration,
            ninja_directory,
        }
    }

    fn compile_main_ninja_file(
        &self,
        package_prefix: &str,
        module_build_targets: &[app::build::ModuleBuildTarget],
    ) -> String {
        vec![
            "ninja_required_version = 1.10".into(),
            "rule compile".into(),
            format!(
                "  command = pen compile -m $module_prefix -p {} $in $out",
                package_prefix
            ),
            "rule compile_dependency".into(),
            "  command = pen compile-dependency $in $out".into(),
        ]
        .into_iter()
        .chain(module_build_targets.iter().flat_map(|target| {
            let input_path = self
                .file_path_converter
                .convert_to_os_path(target.source_file_path());
            let output_path = self
                .file_path_converter
                .convert_to_os_path(target.target_file_path());
            let dependency_path = self
                .file_path_converter
                .convert_to_os_path(&target.target_file_path().with_extension("dd"));

            vec![
                format!(
                    "build {}: compile_dependency {}",
                    dependency_path.display(),
                    input_path.display()
                ),
                format!(
                    "build {}: compile {}",
                    output_path.display(),
                    input_path.display()
                ),
                format!("  dyndep = {}", dependency_path.display()),
            ]
        }))
        .collect::<Vec<String>>()
        .join("\n")
    }

    fn calculate_package_id(&self, package_prefix: &str) -> String {
        let mut hasher = DefaultHasher::new();

        package_prefix.hash(&mut hasher);

        format!("{:x}", hasher.finish())
    }
}

impl app::infra::Builder for Builder {
    fn build(
        &self,
        package_prefix: &str,
        module_build_targets: &[app::build::ModuleBuildTarget],
        output_directory_path: &app::infra::FilePath,
    ) -> Result<(), Box<dyn Error>> {
        let ninja_file_path = output_directory_path
            .join(&app::infra::FilePath::new(vec![
                self.ninja_directory,
                &self.calculate_package_id(package_prefix),
            ]))
            .join(&app::infra::FilePath::new(vec!["build.ninja"]));

        self.file_system.write(
            &ninja_file_path,
            self.compile_main_ninja_file(package_prefix, module_build_targets)
                .as_bytes(),
        )?;

        command_runner::run(
            std::process::Command::new("ninja").arg("-f").arg(
                self.file_path_converter
                    .convert_to_os_path(&app::infra::FilePath::new(vec![
                        self.file_path_configuration.output_directory_name,
                    ])),
            ),
        )?;

        Ok(())
    }
}
