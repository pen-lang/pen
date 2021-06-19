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
            "rule pen_compile",
            "  command = pen compile $in $out",
            "rule pen_compile_dependency",
            "  command = pen compile-dependency -p $package_directory $in $object_file $out",
        ]
        .into_iter()
        .map(String::from)
        .chain(module_targets.iter().flat_map(|target| {
            let package_directory = self
                .file_path_converter
                .convert_to_os_path(target.package_directory());
            let source_path = self
                .file_path_converter
                .convert_to_os_path(target.source_file());
            let interface_path = self
                .file_path_converter
                .convert_to_os_path(target.interface_file());
            let dependency_path = self
                .file_path_converter
                .convert_to_os_path(&target.object_file().with_extension("dd"));
            let object_path = self
                .file_path_converter
                .convert_to_os_path(target.object_file());

            vec![
                format!(
                    "build {}: pen_compile_dependency {}",
                    dependency_path.display(),
                    source_path.display(),
                ),
                format!("  package_directory = {}", package_directory.display()),
                format!("  object_file = {}", object_path.display()),
                format!(
                    "build {} {}: pen_compile {} || {}",
                    object_path.display(),
                    interface_path.display(),
                    source_path.display(),
                    dependency_path.display()
                ),
                format!("  dyndep = {}", dependency_path.display()),
            ]
        }))
        .chain(vec![format!(
            "default {}",
            module_targets
                .iter()
                .map(|target| format!(
                    "{}",
                    self.file_path_converter
                        .convert_to_os_path(target.object_file())
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
        output_directory: &app::infra::FilePath,
    ) -> Result<(), Box<dyn Error>> {
        let ninja_file = output_directory.join(&app::infra::FilePath::new(vec!["build.ninja"]));

        self.file_system.write(
            &ninja_file,
            self.compile_ninja_file(module_targets).as_bytes(),
        )?;

        command_runner::run(
            std::process::Command::new("ninja")
                .arg("-f")
                .arg(self.file_path_converter.convert_to_os_path(&ninja_file)),
        )?;

        Ok(())
    }
}
