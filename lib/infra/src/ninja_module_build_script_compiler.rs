use super::file_path_converter::FilePathConverter;
use app::infra::FilePath;
use std::sync::Arc;

pub struct NinjaModuleBuildScriptCompiler {
    file_path_converter: Arc<FilePathConverter>,
    output_directory: &'static str,
}

impl NinjaModuleBuildScriptCompiler {
    pub fn new(
        file_path_converter: Arc<FilePathConverter>,
        output_directory: &'static str,
    ) -> Self {
        Self {
            file_path_converter,
            output_directory,
        }
    }
}

impl app::infra::ModuleBuildScriptCompiler for NinjaModuleBuildScriptCompiler {
    fn compile(
        &self,
        module_targets: &[app::infra::ModuleTarget],
        sub_build_script_files: &[FilePath],
    ) -> String {
        vec![
            "ninja_required_version = 1.10",
            &format!("builddir = {}", self.output_directory),
            "rule compile",
            "  command = pen compile $in $out",
            "rule resolve_dependency",
            "  command = pen resolve-dependency -p $package_directory $in $object_file $out",
        ]
        .into_iter()
        .map(String::from)
        .chain(sub_build_script_files.iter().map(|file| {
            format!(
                "subninja {}",
                self.file_path_converter.convert_to_os_path(file).display()
            )
        }))
        .chain(module_targets.iter().flat_map(|target| {
            let package_directory = self
                .file_path_converter
                .convert_to_os_path(target.package_directory());
            let source_file = self
                .file_path_converter
                .convert_to_os_path(target.source_file());
            let interface_file = self
                .file_path_converter
                .convert_to_os_path(target.interface_file());
            let dependency_file = self
                .file_path_converter
                .convert_to_os_path(&target.object_file().with_extension("dep"));
            let ninja_dependency_file = self
                .file_path_converter
                .convert_to_os_path(&target.object_file().with_extension("dd"));
            let object_file = self
                .file_path_converter
                .convert_to_os_path(target.object_file());

            vec![
                format!(
                    "build {} {}: compile {} {} || {}",
                    object_file.display(),
                    interface_file.display(),
                    source_file.display(),
                    dependency_file.display(),
                    ninja_dependency_file.display()
                ),
                format!("  dyndep = {}", ninja_dependency_file.display()),
                // TODO Remove this hack to circumvent ninja's bug where dynamic dependency files
                // cannot be specified as inputs together with outputs of the same build rules.
                format!(
                    "build {} {}: resolve_dependency {}",
                    dependency_file.display(),
                    ninja_dependency_file.with_extension("dd.dummy").display(),
                    source_file.display(),
                ),
                format!("  package_directory = {}", package_directory.display()),
                format!("  object_file = {}", object_file.display()),
                format!(
                    "build {} {}: resolve_dependency {}",
                    dependency_file.with_extension("dep.dummy").display(),
                    ninja_dependency_file.display(),
                    source_file.display(),
                ),
                format!("  package_directory = {}", package_directory.display()),
                format!("  object_file = {}", object_file.display()),
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
