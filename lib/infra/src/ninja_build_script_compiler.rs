use super::file_path_converter::FilePathConverter;
use crate::{default_target_finder, llvm_command_finder, InfrastructureError};
use app::infra::FilePath;
use std::{error::Error, sync::Arc};

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

    fn compile_rules(
        &self,
        prelude_interface_files: &[FilePath],
        target_triple: Option<&str>,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let llc = llvm_command_finder::find("llc")?;

        let resolve_dependency_command = format!(
            "  command = pen resolve-dependency -o $builddir -p $package_directory {} $in $object_file $out",
            prelude_interface_files
                .iter()
                .map(|file| "-i ".to_owned()
                    + &self
                        .file_path_converter
                        .convert_to_os_path(file)
                        .display()
                        .to_string())
                .collect::<Vec<_>>()
                .join(" "),
        );

        Ok([
            &format!(
                "target = {}",
                if let Some(triple) = target_triple {
                    triple.into()
                } else {
                    default_target_finder::find()?
                }
            ),
            "rule compile",
            "  command = pen compile --target $target $in $out",
            "  description = compiling module of $source_file",
            "rule compile_main",
            "  command = pen compile-main --target $target \
                -f $main_function_interface_file $in $out",
            "  description = compiling module of $source_file",
            "rule compile_prelude",
            "  command = pen compile-prelude --target $target $in $out",
            "  description = compiling module of $source_file",
            "rule llc",
            &format!(
                "  command = {} -O3 -tailcallopt --relocation-model pic \
                    -mtriple $target -filetype obj -o $out $in",
                llc.display()
            ),
            "  description = generating object file for $source_file",
            "rule resolve_dependency",
            &resolve_dependency_command,
            "  description = resolving dependency of $in",
            "rule compile_ffi",
            "  command = $in -t $target $out",
        ]
        .iter()
        .map(|string| string.to_string())
        .collect())
    }

    fn compile_module_targets(
        &self,
        module_targets: &[app::infra::ModuleTarget],
        ffi_archive_file: &FilePath,
        package_directory: &FilePath,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        Ok(module_targets
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
                let object_file = self
                    .file_path_converter
                    .convert_to_os_path(target.object_file());
                let dependency_file = object_file.with_extension("dep");
                let ninja_dependency_file = object_file.with_extension("dd");
                let bit_code_file = object_file.with_extension(self.bit_code_file_extension);

                vec![
                    format!(
                        "build {} {}: compile {} {} || {}",
                        bit_code_file.display(),
                        interface_file.display(),
                        source_file.display(),
                        dependency_file.display(),
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
                ]
                .into_iter()
                .chain(self.compile_dependency(
                    &source_file,
                    &bit_code_file,
                    &dependency_file,
                    &ninja_dependency_file,
                    &package_directory,
                ))
            })
            .chain(if module_targets.is_empty() {
                None
            } else {
                Some(format!(
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
                ))
            })
            .chain(self.compile_ffi_build(package_directory, ffi_archive_file)?)
            .collect())
    }

    // TODO Remove this hack to circumvent ninja's bug where dynamic dependency files
    // cannot be specified as inputs together with outputs of the same build rules.
    // https://github.com/ninja-build/ninja/issues/1988
    fn compile_dependency(
        &self,
        source_file: &std::path::Path,
        bit_code_file: &std::path::Path,
        dependency_file: &std::path::Path,
        ninja_dependency_file: &std::path::Path,
        package_directory: &std::path::Path,
    ) -> Vec<String> {
        vec![
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
    }

    fn compile_ffi_build(
        &self,
        package_directory: &FilePath,
        archive_file: &FilePath,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let package_directory = self
            .file_path_converter
            .convert_to_os_path(package_directory);
        let ffi_build_scripts = glob::glob(
            &(package_directory
                .join(self.ffi_build_script)
                .to_string_lossy()
                + ".*"),
        )?
        .collect::<Vec<_>>()
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
        let archive_file = self.file_path_converter.convert_to_os_path(archive_file);

        Ok(match ffi_build_scripts.as_slice() {
            [] => vec![],
            [script] => vec![
                format!(
                    "build {}: compile_ffi {}",
                    archive_file.display(),
                    script.display()
                ),
                format!("default {}", archive_file.display()),
            ],
            _ => return Err(InfrastructureError::TooManyFfiBuildScripts(package_directory).into()),
        })
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
        target_triple: Option<&str>,
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
        .chain(self.compile_rules(prelude_interface_files, target_triple)?)
        .chain(self.compile_module_targets(module_targets, ffi_archive_file, package_directory)?)
        .chain(if let Some(main_module_target) = main_module_target {
            let source_file = self
                .file_path_converter
                .convert_to_os_path(main_module_target.source_file());
            let object_file = self
                .file_path_converter
                .convert_to_os_path(main_module_target.object_file());
            let dependency_file = object_file.with_extension("dep");
            let ninja_dependency_file = object_file.with_extension("dd");
            let bit_code_file = object_file.with_extension(self.bit_code_file_extension);
            let main_function_interface_file = self
                .file_path_converter
                .convert_to_os_path(main_module_target.main_function_interface_file());

            vec![
                format!(
                    "build {}: compile_main {} {} | {} || {}",
                    bit_code_file.display(),
                    source_file.display(),
                    dependency_file.display(),
                    main_function_interface_file.display(),
                    ninja_dependency_file.display(),
                ),
                format!(
                    "  main_function_interface_file = {}",
                    main_function_interface_file.display()
                ),
                format!("  dyndep = {}", ninja_dependency_file.display()),
                format!("  source_file = {}", source_file.display()),
                format!(
                    "build {}: llc {}",
                    object_file.display(),
                    bit_code_file.display(),
                ),
                format!("  source_file = {}", source_file.display()),
                format!("default {}", object_file.display()),
            ]
            .into_iter()
            .chain(
                self.compile_dependency(
                    &source_file,
                    &bit_code_file,
                    &dependency_file,
                    &ninja_dependency_file,
                    &self
                        .file_path_converter
                        .convert_to_os_path(package_directory),
                ),
            )
            .collect()
        } else {
            vec![]
        })
        .chain(child_build_script_files.iter().map(|file| {
            format!(
                "subninja {}",
                self.file_path_converter.convert_to_os_path(file).display()
            )
        }))
        .collect::<Vec<_>>()
        .join("\n")
            + "\n")
    }

    fn compile_external(
        &self,
        module_targets: &[app::infra::ModuleTarget],
        ffi_archive_file: &FilePath,
        package_directory: &FilePath,
    ) -> Result<String, Box<dyn Error>> {
        Ok(self
            .compile_module_targets(module_targets, ffi_archive_file, package_directory)?
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
            .chain(self.compile_ffi_build(package_directory, ffi_archive_file)?)
            .collect::<Vec<String>>()
            .join("\n")
            + "\n")
    }
}
