use super::file_path_converter::FilePathConverter;
use app::infra::FilePath;
use std::{error::Error, path::PathBuf, sync::Arc};

const APPLICATION_EXECUTABLE_FILENAME: &str = "app";

pub struct NinjaBuildScriptCompiler {
    file_path_converter: Arc<FilePathConverter>,
    bit_code_file_extension: &'static str,
    ffi_build_script: &'static str,
}

impl NinjaBuildScriptCompiler {
    pub fn new(
        file_path_converter: Arc<FilePathConverter>,
        bit_code_file_extension: &'static str,
        ffi_build_script: &'static str,
    ) -> Self {
        Self {
            file_path_converter,
            bit_code_file_extension,
            ffi_build_script,
        }
    }

    fn compile_rules(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let llc = self.find_llc()?;

        Ok([
            "rule compile",
            "  command = pen compile $in $out",
            "  description = compiling module of $source_file",
            "rule compile_prelude",
            "  command = pen compile-prelude $in $out",
            "  description = compiling module of $source_file",
            "rule llc",
            &format!(
                "  command = {} -O3 -tailcallopt -filetype obj -o $out $in",
                llc.display()
            ),
            "  description = generating object file for $source_file",
            "rule resolve_dependency",
            "  command = pen resolve-dependency -o $builddir -p $package_directory $in $object_file $out",
            "  description = resolving dependency of $in",
            "rule compile_ffi",
            "  command = $in $out",
        ]
        .iter()
        .map(|string| string.to_string())
        .collect())
    }

    fn compile_common(
        &self,
        module_targets: &[app::infra::ModuleTarget],
        prelude_interface_files: &[FilePath],
        ffi_archive_file: &FilePath,
        package_directory: &FilePath,
    ) -> Vec<String> {
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

        module_targets
            .iter()
            .flat_map(|target| {
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
            })
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
            .chain(self.compile_ffi_build(package_directory, ffi_archive_file))
            .collect()
    }

    fn find_llc(&self) -> Result<PathBuf, Box<dyn Error>> {
        Ok(which::which("llc-13")
            .or_else(|_| which::which("llc-12"))
            .or_else(|_| which::which("llc-11"))
            .or_else(|_| which::which("llc"))?)
    }

    fn compile_ffi_build(
        &self,
        package_directory: &FilePath,
        archive_file: &FilePath,
    ) -> Vec<String> {
        let package_directory = self
            .file_path_converter
            .convert_to_os_path(package_directory);
        let ffi_build_script = package_directory.join(self.ffi_build_script);
        let archive_file = self.file_path_converter.convert_to_os_path(archive_file);

        if ffi_build_script.exists() {
            vec![
                format!(
                    "build {}: compile_ffi {}",
                    archive_file.display(),
                    ffi_build_script.display()
                ),
                format!("default {}", archive_file.display()),
            ]
        } else {
            vec![]
        }
    }
}

impl app::infra::BuildScriptCompiler for NinjaBuildScriptCompiler {
    fn compile_main(
        &self,
        module_targets: &[app::infra::ModuleTarget],
        main_module_target: Option<&app::infra::MainModuleTarget>,
        child_build_script_files: &[FilePath],
        prelude_interface_files: &[FilePath],
        ffi_archive_file: &FilePath,
        package_directory: &FilePath,
        output_directory: &FilePath,
    ) -> Result<String, Box<dyn Error>> {
        Ok(vec![
            "ninja_required_version = 1.10".into(),
            format!(
                "builddir = {}",
                self.file_path_converter
                    .convert_to_os_path(output_directory)
                    .display()
            ),
        ]
        .into_iter()
        .chain(self.compile_rules()?)
        .chain(self.compile_common(
            module_targets,
            prelude_interface_files,
            ffi_archive_file,
            package_directory,
        ))
        .chain(child_build_script_files.iter().map(|file| {
            format!(
                "subninja {}",
                self.file_path_converter.convert_to_os_path(file).display()
            )
        }))
        .chain(if let Some(main_module_target) = main_module_target {
            // TODO Collect those from arguments.
            let object_files = module_targets
                .iter()
                .map(|target| target.object_file())
                .map(|path| {
                    self.file_path_converter
                        .convert_to_os_path(path)
                        .display()
                        .to_string()
                })
                .collect::<Vec<String>>();

            let application_file = self.file_path_converter.convert_to_os_path(
                &package_directory.join(&FilePath::new([APPLICATION_EXECUTABLE_FILENAME])),
            );

            vec![
                "rule compile_main".into(),
                format!(
                    "  command = ein compile-main -s {} $in $out",
                    self.file_path_converter
                        .convert_to_os_path(main_module_target.system_package_directory())
                        .display()
                ),
                "  description = compiling main module".into(),
                "rule link".into(),
                // spell-checker: disable-next-line
                "  command = clang -Werror -o $out $in -ldl -lpthread".into(),
                "  description = linking application file".into(),
                format!(
                    "build {}: compile_main {} {}",
                    self.file_path_converter
                        .convert_to_os_path(main_module_target.object_file())
                        .display(),
                    self.file_path_converter
                        .convert_to_os_path(main_module_target.source_file())
                        .display(),
                    self.file_path_converter
                        .convert_to_os_path(&main_module_target.object_file().with_extension("dep"))
                        .display(),
                ),
                format!(
                    "build {}: link {}",
                    application_file.display(),
                    object_files.join(" ")
                ),
                format!("default {}", application_file.display()),
            ]
        } else {
            vec![]
        })
        .collect::<Vec<_>>()
        .join("\n")
            + "\n")
    }

    fn compile_external(
        &self,
        module_targets: &[app::infra::ModuleTarget],
        prelude_interface_files: &[FilePath],
        ffi_archive_file: &FilePath,
        package_directory: &FilePath,
    ) -> Result<String, Box<dyn Error>> {
        Ok(self
            .compile_common(
                module_targets,
                prelude_interface_files,
                ffi_archive_file,
                package_directory,
            )
            .join("\n")
            + "\n")
    }

    fn compile_prelude(
        &self,
        module_targets: &[app::infra::ModuleTarget],
        ffi_archive_file: &FilePath,
        package_directory: &FilePath,
    ) -> Result<String, Box<dyn Error>> {
        Ok(module_targets
            .iter()
            .flat_map(|target| {
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
                        "build {} {}: compile_prelude {}",
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
            })
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
            .chain(self.compile_ffi_build(package_directory, ffi_archive_file))
            .collect::<Vec<String>>()
            .join("\n")
            + "\n")
    }
}
