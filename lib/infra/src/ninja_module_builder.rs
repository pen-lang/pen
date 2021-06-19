use super::{command_runner, file_path_converter::FilePathConverter};
use std::{error::Error, sync::Arc};

pub struct NinjaModuleBuilder {
    file_system: Arc<dyn app::infra::FileSystem>,
    file_path_converter: Arc<FilePathConverter>,
}

impl NinjaModuleBuilder {
    pub fn new(
        file_system: Arc<dyn app::infra::FileSystem>,
        file_path_converter: Arc<FilePathConverter>,
    ) -> Self {
        Self {
            file_system,
            file_path_converter,
        }
    }

    fn compile_ninja_file(&self, module_targets: &[app::build::ModuleTarget]) -> String {
        vec![
            "ninja_required_version = 1.10",
            "rule compile",
            "  command = pen compile -p $package_prefix -m $module_prefix $in $out",
            "rule compile_dependency",
            "  command = pen compile-dependency $in $out",
        ]
        .into_iter()
        .map(String::from)
        .chain(module_targets.iter().flat_map(|target| {
            let input_path = self
                .file_path_converter
                .convert_to_os_path(target.source_file_path());
            let dependency_path = self
                .file_path_converter
                .convert_to_os_path(&target.object_file_path().with_extension("dd"));

            vec![
                format!(
                    "build {}: compile_dependency {}",
                    dependency_path.display(),
                    input_path.display()
                ),
                format!(
                    "build {}: compile {} || {}",
                    self.file_path_converter
                        .convert_to_os_path(target.object_file_path())
                        .display(),
                    input_path.display(),
                    dependency_path.display()
                ),
                format!("  dyndep = {}", dependency_path.display()),
                format!("  package_prefix = {}", target.package_prefix()),
                format!("  module_prefix = {}", target.module_prefix()),
            ]
        }))
        .chain(vec![format!(
            "default {}",
            module_targets
                .iter()
                .map(|target| format!(
                    "{}",
                    self.file_path_converter
                        .convert_to_os_path(target.object_file_path())
                        .display()
                ))
                .collect::<Vec<_>>()
                .join(" ")
        )])
        .collect::<Vec<String>>()
        .join("\n")
            + "\n"
    }
}

impl app::infra::ModuleBuilder for NinjaModuleBuilder {
    fn build(
        &self,
        module_targets: &[app::build::ModuleTarget],
        output_directory_path: &app::infra::FilePath,
    ) -> Result<(), Box<dyn Error>> {
        let ninja_file_path =
            output_directory_path.join(&app::infra::FilePath::new(vec!["build.ninja"]));

        self.file_system.write(
            &ninja_file_path,
            self.compile_ninja_file(module_targets).as_bytes(),
        )?;

        command_runner::run(
            std::process::Command::new("ninja").arg("-f").arg(
                self.file_path_converter
                    .convert_to_os_path(&ninja_file_path),
            ),
        )?;

        Ok(())
    }
}
