use super::file_path_converter::FilePathConverter;
use crate::{
    default_target_finder, llvm_command_finder, package_script_finder, FilePathDisplayer,
    InfrastructureError,
};
use app::infra::{FilePath, FilePathDisplayer as _};
use std::{error::Error, sync::Arc};

pub struct NinjaBuildScriptCompiler {
    file_path_converter: Arc<FilePathConverter>,
    file_path_displayer: Arc<FilePathDisplayer>,
    bit_code_file_extension: &'static str,
    ffi_build_script_basename: &'static str,
    link_script_basename: &'static str,
}

impl NinjaBuildScriptCompiler {
    pub fn new(
        file_path_converter: Arc<FilePathConverter>,
        file_path_displayer: Arc<FilePathDisplayer>,
        bit_code_file_extension: &'static str,
        ffi_build_script_basename: &'static str,
        link_script_basename: &'static str,
    ) -> Self {
        Self {
            file_path_converter,
            file_path_displayer,
            bit_code_file_extension,
            ffi_build_script_basename,
            link_script_basename,
        }
    }

    fn compile_rules(
        &self,
        prelude_interface_files: &[FilePath],
        target_triple: Option<&str>,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let llc = llvm_command_finder::find("llc")?;
        let ar = llvm_command_finder::find("llvm-ar")?;

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
            "rule compile_test",
            "  command = pen compile-test --target $target $in $out",
            "  description = compiling module of $source_file",
            "rule compile_package_test_information",
            "  command = pen compile-package-test-information -o $out $in",
            "rule llc",
            &format!(
                "  command = {} -O3 -tailcallopt --relocation-model pic \
                    -mtriple $target {} -filetype obj -o $out $in",
                llc.display(),
                if target_triple
                    .map(|target| target.starts_with("wasm"))
                    .unwrap_or_default()
                {
                    // spell-checker: disable-next-line
                    "-mattr +tail-call"
                } else {
                    ""
                }
            ),
            "rule resolve_dependency",
            &resolve_dependency_command,
            "  description = resolving dependency of $source_file",
            "rule ar",
            &format!("  command = {} crs $out $in", ar.display()),
            "  description = archiving package at $package_directory",
            "rule compile_ffi",
            "  command = $in -t $target $out",
            "  description = compiling FFI module at $package_directory",
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
                    format!(
                        "  source_file = {}",
                        self.file_path_displayer.display(target.source_file())
                    ),
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
                    target.source_file(),
                ))
            })
            .chain(self.compile_ffi_build(package_directory, ffi_archive_file)?)
            .collect())
    }

    fn compile_test_module_targets(
        &self,
        module_targets: &[app::infra::TestModuleTarget],
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
                let test_information_file = self
                    .file_path_converter
                    .convert_to_os_path(target.test_information_file());
                let object_file = self
                    .file_path_converter
                    .convert_to_os_path(target.object_file());
                let dependency_file = object_file.with_extension("dep");
                let ninja_dependency_file = object_file.with_extension("dd");
                let bit_code_file = object_file.with_extension(self.bit_code_file_extension);

                vec![
                    format!(
                        "build {} {}: compile_test {} {} || {}",
                        bit_code_file.display(),
                        test_information_file.display(),
                        source_file.display(),
                        dependency_file.display(),
                        ninja_dependency_file.display()
                    ),
                    format!("  dyndep = {}", ninja_dependency_file.display()),
                    format!(
                        "  source_file = {}",
                        self.file_path_displayer.display(target.source_file())
                    ),
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
                    target.source_file(),
                ))
            })
            .collect())
    }

    fn compile_main_module_target(
        &self,
        target: &app::infra::MainModuleTarget,
        package_directory: &FilePath,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let source_file = self
            .file_path_converter
            .convert_to_os_path(target.source_file());
        let object_file = self
            .file_path_converter
            .convert_to_os_path(target.object_file());
        let dependency_file = object_file.with_extension("dep");
        let ninja_dependency_file = object_file.with_extension("dd");
        let bit_code_file = object_file.with_extension(self.bit_code_file_extension);
        let main_function_interface_file = self
            .file_path_converter
            .convert_to_os_path(target.main_function_interface_file());

        Ok(vec![
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
            format!(
                "  source_file = {}",
                self.file_path_displayer.display(target.source_file())
            ),
            format!(
                "build {}: llc {}",
                object_file.display(),
                bit_code_file.display(),
            ),
            format!("  source_file = {}", source_file.display()),
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
                target.source_file(),
            ),
        )
        .collect())
    }

    fn compile_dependency(
        &self,
        source_file: &std::path::Path,
        bit_code_file: &std::path::Path,
        dependency_file: &std::path::Path,
        ninja_dependency_file: &std::path::Path,
        package_directory: &std::path::Path,
        original_source_file: &FilePath,
    ) -> Vec<String> {
        vec![
            format!(
                "build {} {}: resolve_dependency {}",
                dependency_file.display(),
                ninja_dependency_file.display(),
                source_file.display(),
            ),
            format!("  package_directory = {}", package_directory.display()),
            format!("  object_file = {}", bit_code_file.display()),
            format!(
                "  source_file = {}",
                self.file_path_displayer.display(original_source_file)
            ),
        ]
    }

    fn compile_ffi_build(
        &self,
        original_package_directory: &FilePath,
        archive_file: &FilePath,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let package_directory = self
            .file_path_converter
            .convert_to_os_path(original_package_directory);

        Ok(
            if let Some(script) =
                package_script_finder::find(&package_directory, self.ffi_build_script_basename)?
            {
                let archive_file = self.file_path_converter.convert_to_os_path(archive_file);

                vec![
                    format!(
                        "build {}: compile_ffi {}",
                        archive_file.display(),
                        script.display()
                    ),
                    format!(
                        "  package_directory = {}",
                        self.file_path_displayer.display(original_package_directory)
                    ),
                    format!("default {}", archive_file.display()),
                ]
            } else {
                vec![]
            },
        )
    }

    fn compile_archive(
        &self,
        object_files: &[&FilePath],
        archive_file: &FilePath,
        package_directory: &FilePath,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let archive_file = self.file_path_converter.convert_to_os_path(archive_file);

        Ok(vec![
            format!(
                "build {}: ar {}",
                archive_file.display(),
                object_files
                    .iter()
                    .map(|object_file| format!(
                        "{}",
                        self.file_path_converter
                            .convert_to_os_path(object_file)
                            .display()
                    ))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            format!(
                "  package_directory = {}",
                self.file_path_displayer.display(package_directory)
            ),
            format!("default {}", archive_file.display()),
        ])
    }

    fn compile_package_test_information(
        &self,
        test_information_files: &[&FilePath],
        package_test_information_file: &FilePath,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let package_test_information_file = self
            .file_path_converter
            .convert_to_os_path(package_test_information_file);

        Ok(vec![
            format!(
                "build {}: compile_package_test_information {}",
                package_test_information_file.display(),
                test_information_files
                    .iter()
                    .map(|file| format!(
                        "{}",
                        self.file_path_converter.convert_to_os_path(file).display()
                    ))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            format!("default {}", package_test_information_file.display()),
        ])
    }
}

impl app::infra::BuildScriptCompiler for NinjaBuildScriptCompiler {
    fn compile_main(
        &self,
        prelude_interface_files: &[FilePath],
        output_directory: &FilePath,
        target_triple: Option<&str>,
        child_build_script_files: &[FilePath],
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

    fn compile_modules(
        &self,
        module_targets: &[app::infra::ModuleTarget],
        main_module_target: Option<&app::infra::MainModuleTarget>,
        archive_file: &FilePath,
        ffi_archive_file: &FilePath,
        package_directory: &FilePath,
    ) -> Result<String, Box<dyn Error>> {
        Ok(self
            .compile_module_targets(module_targets, ffi_archive_file, package_directory)?
            .into_iter()
            .chain(if let Some(main_module_target) = main_module_target {
                self.compile_main_module_target(main_module_target, package_directory)?
            } else {
                vec![]
            })
            .chain(
                self.compile_archive(
                    &module_targets
                        .iter()
                        .map(|target| target.object_file())
                        .chain(main_module_target.map(|target| target.object_file()))
                        .collect::<Vec<_>>(),
                    archive_file,
                    package_directory,
                )?,
            )
            .collect::<Vec<_>>()
            .join("\n")
            + "\n")
    }

    fn compile_test_modules(
        &self,
        module_targets: &[app::infra::TestModuleTarget],
        archive_file: &FilePath,
        package_test_information_file: &FilePath,
        package_directory: &FilePath,
    ) -> Result<String, Box<dyn Error>> {
        Ok(self
            .compile_test_module_targets(module_targets)?
            .into_iter()
            .chain(
                self.compile_archive(
                    &module_targets
                        .iter()
                        .map(|target| target.object_file())
                        .collect::<Vec<_>>(),
                    archive_file,
                    package_directory,
                )?,
            )
            .chain(
                self.compile_package_test_information(
                    &module_targets
                        .iter()
                        .map(|target| target.test_information_file())
                        .collect::<Vec<_>>(),
                    package_test_information_file,
                )?,
            )
            .collect::<Vec<_>>()
            .join("\n")
            + "\n")
    }

    fn compile_external(
        &self,
        module_targets: &[app::infra::ModuleTarget],
        archive_file: &FilePath,
        ffi_archive_file: &FilePath,
        package_directory: &FilePath,
    ) -> Result<String, Box<dyn Error>> {
        Ok(self
            .compile_module_targets(module_targets, ffi_archive_file, package_directory)?
            .into_iter()
            .chain(
                self.compile_archive(
                    &module_targets
                        .iter()
                        .map(|target| target.object_file())
                        .collect::<Vec<_>>(),
                    archive_file,
                    package_directory,
                )?,
            )
            .collect::<Vec<_>>()
            .join("\n")
            + "\n")
    }

    fn compile_application(
        &self,
        system_package_directory: &FilePath,
        archive_files: &[FilePath],
        application_file: &FilePath,
    ) -> Result<String, Box<dyn Error>> {
        let system_package_directory = self
            .file_path_converter
            .convert_to_os_path(system_package_directory);
        let application_file = self
            .file_path_converter
            .convert_to_os_path(application_file);

        Ok(vec![
            "rule link".into(),
            format!(
                "  command = {} -t $target -o $out $in",
                package_script_finder::find(&system_package_directory, self.link_script_basename)?
                    .ok_or(InfrastructureError::LinkScriptNotFound(
                        system_package_directory,
                    ))?
                    .display(),
            ),
            "  description = linking application".into(),
            format!(
                "build {}: link {}",
                application_file.display(),
                archive_files
                    .iter()
                    .map(|file| self
                        .file_path_converter
                        .convert_to_os_path(file)
                        .display()
                        .to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            format!("default {}", application_file.display()),
        ]
        .join("\n")
            + "\n")
    }

    fn compile_test(
        &self,
        archive_files: &[FilePath],
        test_information_file: &FilePath,
        test_file: &FilePath,
    ) -> Result<String, Box<dyn Error>> {
        let test_file = self.file_path_converter.convert_to_os_path(test_file);

        Ok(vec![
            "rule link".into(),
            "  command = pen link-test -o $out -i $in".into(),
            "  description = linking tests".into(),
            format!(
                "build {}: link {} {}",
                test_file.display(),
                self.file_path_converter
                    .convert_to_os_path(test_information_file)
                    .display(),
                archive_files
                    .iter()
                    .map(|file| self
                        .file_path_converter
                        .convert_to_os_path(file)
                        .display()
                        .to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            format!("default {}", test_file.display()),
        ]
        .join("\n")
            + "\n")
    }

    fn compile_prelude(
        &self,
        module_targets: &[app::infra::ModuleTarget],
        archive_file: &FilePath,
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
            .chain(
                self.compile_archive(
                    &module_targets
                        .iter()
                        .map(|target| target.object_file())
                        .collect::<Vec<_>>(),
                    archive_file,
                    package_directory,
                )?,
            )
            .chain(self.compile_ffi_build(package_directory, ffi_archive_file)?)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n")
    }
}
