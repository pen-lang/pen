use super::file_path_converter::FilePathConverter;
use app::infra::FilePath;
use std::{error::Error, path::PathBuf, sync::Arc};

pub struct NinjaModuleBuildScriptCompiler {
    file_path_converter: Arc<FilePathConverter>,
    bit_code_file_extension: &'static str,
    log_directory: &'static str,
}

impl NinjaModuleBuildScriptCompiler {
    pub fn new(
        file_path_converter: Arc<FilePathConverter>,
        bit_code_file_extension: &'static str,
        log_directory: &'static str,
    ) -> Self {
        Self {
            file_path_converter,
            bit_code_file_extension,
            log_directory,
        }
    }

    fn find_llc(&self) -> Result<PathBuf, Box<dyn Error>> {
        Ok(which::which("llc-13")
            .or_else(|_| which::which("llc-12"))
            .or_else(|_| which::which("llc-11"))
            .or_else(|_| which::which("llc"))?)
    }
}

impl app::infra::ModuleBuildScriptCompiler for NinjaModuleBuildScriptCompiler {
    fn compile(
        &self,
        package_directory: &FilePath,
        module_targets: &[app::infra::ModuleTarget],
        child_build_script_files: &[FilePath],
        prelude_interface_files: &[FilePath],
    ) -> Result<String, Box<dyn Error>> {
        let llc = self.find_llc()?;
        let prelude_interface_files_string = prelude_interface_files
            .iter()
            .map(|file| {
                self.file_path_converter
                    .convert_to_os_path(file)
                    .display()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join(" ");

        Ok(vec![
            "ninja_required_version = 1.10",
            &format!("builddir = {}", self.log_directory),
            "rule compile",
            "  command = pen compile $in $out",
            "  description = compiling module of $source_file",
            "rule llc",
            &format!(
                "  command = {} -O3 -tailcallopt -filetype obj -o $out $in",
                llc.display()
            ),
            "  description = generating object file for $source_file",
            "rule resolve_dependency",
            "  command = pen resolve-dependency -p $package_directory $in $object_file $out",
            "  description = resolving dependency of $in",
        ]
        .into_iter()
        .map(String::from)
        .chain(child_build_script_files.iter().map(|file| {
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
            let bit_code_file = object_file.with_extension(self.bit_code_file_extension);

            vec![
                format!(
                    "build {} {}: compile {} {} | {} || {}",
                    bit_code_file.display(),
                    interface_file.display(),
                    source_file.display(),
                    dependency_file.display(),
                    prelude_interface_files_string,
                    ninja_dependency_file.display()
                ),
                format!("  dyndep = {}", ninja_dependency_file.display()),
                format!("  source_file = {}", source_file.display()),
                format!(
                    "build {}: llc {}",
                    object_file.display(),
                    bit_code_file.display(),
                ),
                format!("  source_file = {}", source_file.display()),
                // TODO Remove this hack to circumvent ninja's bug where dynamic dependency files
                // cannot be specified as inputs together with outputs of the same build rules.
                // https://github.com/ninja-build/ninja/issues/1988
                format!(
                    "build {} {}: resolve_dependency {}",
                    dependency_file.display(),
                    ninja_dependency_file.with_extension("dd.dummy").display(),
                    source_file.display(),
                ),
                format!("  package_directory = {}", package_directory.display()),
                format!("  object_file = {}", bit_code_file.display()),
                format!(
                    "build {} {}: resolve_dependency {}",
                    dependency_file.with_extension("dep.dummy").display(),
                    ninja_dependency_file.display(),
                    source_file.display(),
                ),
                format!("  package_directory = {}", package_directory.display()),
                format!("  object_file = {}", bit_code_file.display()),
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
            + "\n")
    }

    fn compile_prelude(
        &self,
        module_targets: &[app::infra::ModuleTarget],
    ) -> Result<String, Box<dyn Error>> {
        let llc = self.find_llc()?;

        Ok(vec![
            "ninja_required_version = 1.10",
            &format!("builddir = {}", self.log_directory),
            "rule compile",
            "  command = pen compile-prelude $in $out",
            "  description = compiling module of $source_file",
            "rule llc",
            &format!(
                "  command = {} -O3 -tailcallopt -filetype obj -o $out $in",
                llc.display()
            ),
            "  description = generating object file for $source_file",
        ]
        .into_iter()
        .map(String::from)
        .chain(module_targets.iter().flat_map(|target| {
            let source_file = self
                .file_path_converter
                .convert_to_os_path(target.source_file());
            let interface_file = self
                .file_path_converter
                .convert_to_os_path(target.interface_file());
            let object_file = self
                .file_path_converter
                .convert_to_os_path(target.object_file());
            let bit_code_file = object_file.with_extension(self.bit_code_file_extension);

            vec![
                format!(
                    "build {} {}: compile {}",
                    bit_code_file.display(),
                    interface_file.display(),
                    source_file.display(),
                ),
                format!("  source_file = {}", source_file.display()),
                format!(
                    "build {}: llc {}",
                    object_file.display(),
                    bit_code_file.display(),
                ),
                format!("  source_file = {}", source_file.display()),
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
            + "\n")
    }
}
