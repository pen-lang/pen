use super::file_path_converter::FilePathConverter;
use crate::{
    default_target_finder, ffi_crate_finder, llvm_command_finder, package_script_finder,
    InfrastructureError,
};
use app::infra::FilePath;
use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    path::{Component, Path, PathBuf},
    rc::Rc,
};

const FFI_ARCHIVE_DIRECTORY: &str = "ffi";
const FFI_PHONY_TARGET: &str = "ffi";
const FFI_BUILD_STAMP: &str = "ffi_build.stamp";
const AR_DESCRIPTION: &str = "  description = archiving package $package_name";

pub struct NinjaBuildScriptCompiler {
    file_path_converter: Rc<FilePathConverter>,
    bit_code_file_extension: &'static str,
    dependency_file_extension: &'static str,
    ninja_dynamic_dependency_file_extension: &'static str,
    link_script_basename: &'static str,
}

impl NinjaBuildScriptCompiler {
    pub fn new(
        file_path_converter: Rc<FilePathConverter>,
        bit_code_file_extension: &'static str,
        dependency_file_extension: &'static str,
        ninja_dynamic_dependency_file_extension: &'static str,
        link_script_basename: &'static str,
    ) -> Self {
        Self {
            file_path_converter,
            bit_code_file_extension,
            dependency_file_extension,
            ninja_dynamic_dependency_file_extension,
            link_script_basename,
        }
    }

    fn compile_rules(
        &self,
        prelude_interface_files: &[FilePath],
        target_triple: Option<&str>,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let llc = llvm_command_finder::find("llc")?;
        let opt = llvm_command_finder::find("opt")?;
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
            "  description = compiling module $module_name $in_package_name",
            "rule compile_main",
            "  command = pen compile-main --target $target \
                 $context_options $in $out",
            "  description = compiling module $module_name",
            "rule compile_prelude",
            "  command = pen compile-prelude --target $target $in $out",
            "rule compile_test",
            "  command = pen compile-test --target $target $in $out",
            "  description = compiling test module $module_name",
            "rule compile_package_test_information",
            "  command = pen compile-package-test-information -o $out $in",
            "rule opt",
            // Do not use the -sccp pass here as it breaks tail call optimization by llc because we
            // use a return type of an empty struct for CPS!
            //
            // TODO Use a void type as a return type in CPS.
            // spell-checker: disable
            &format!(
                "  command = {} \
                    -p verify,function-attrs,globalopt,adce,instcombine,tailcallelim,inline,mergefunc,verify \
                    -o $out $in",
                opt.display(),
            ),
            // spell-checker: enable
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
            "  description = resolving dependency of module $module_name $in_package_name",
            "rule build_ffi",
            "  command = cargo build --manifest-path $manifest_path --release --quiet --target $target && touch $out",
            "  description = building FFI workspace",
            "rule copy_ffi",
            "  command = cp $source $out",
            "  description = copying FFI archive $package_name",
            "rule ar",
            &format!("  command = {} crs $out $in", ar.display()),
            AR_DESCRIPTION,
            "rule ar_ffi",
            &format!(
                "  command = cp $ffi_archive_file $out && {} crs $out $object_files",
                ar.display()
            ),
            AR_DESCRIPTION,
        ]
        .iter()
        .map(|string| string.to_string())
        .collect())
    }

    fn compile_module_targets(
        &self,
        module_targets: &[app::infra::ModuleTarget],
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
                let dependency_file = object_file.with_extension(self.dependency_file_extension);
                let ninja_dependency_file =
                    object_file.with_extension(self.ninja_dynamic_dependency_file_extension);
                let bit_code_file = object_file.with_extension(self.bit_code_file_extension);

                [
                    format!(
                        "build {} {}: compile {} {} || {}",
                        bit_code_file.display(),
                        interface_file.display(),
                        source_file.display(),
                        dependency_file.display(),
                        ninja_dependency_file.display()
                    ),
                    format!("  dyndep = {}", ninja_dependency_file.display()),
                    format!("  srcdep = {}", target.source_file()),
                    format!("  module_name = {}", target.source().module_name()),
                    self.format_in_package_name_variable(target.source().package_name()),
                ]
                .into_iter()
                .chain(self.compile_object_file(&bit_code_file, &object_file))
                .chain(self.compile_dependency(
                    &source_file,
                    &bit_code_file,
                    &dependency_file,
                    &ninja_dependency_file,
                    &package_directory,
                    target.source_file(),
                    target.source(),
                ))
            })
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
                let dependency_file = object_file.with_extension(self.dependency_file_extension);
                let ninja_dependency_file =
                    object_file.with_extension(self.ninja_dynamic_dependency_file_extension);
                let bit_code_file = object_file.with_extension(self.bit_code_file_extension);

                [
                    format!(
                        "build {} {}: compile_test {} {} || {}",
                        bit_code_file.display(),
                        test_information_file.display(),
                        source_file.display(),
                        dependency_file.display(),
                        ninja_dependency_file.display()
                    ),
                    format!("  dyndep = {}", ninja_dependency_file.display()),
                    format!("  module_name = {}", target.source().module_name()),
                    format!("  srcdep = {}", target.source_file()),
                ]
                .into_iter()
                .chain(self.compile_object_file(&bit_code_file, &object_file))
                .chain(self.compile_dependency(
                    &source_file,
                    &bit_code_file,
                    &dependency_file,
                    &ninja_dependency_file,
                    &package_directory,
                    target.source_file(),
                    target.source(),
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
        let dependency_file = object_file.with_extension(self.dependency_file_extension);
        let ninja_dependency_file =
            object_file.with_extension(self.ninja_dynamic_dependency_file_extension);
        let bit_code_file = object_file.with_extension(self.bit_code_file_extension);
        let context_interface_files = target
            .context_interface_files()
            .iter()
            .map(|(key, path)| {
                (
                    key.clone(),
                    self.file_path_converter.convert_to_os_path(path),
                )
            })
            .collect::<BTreeMap<_, _>>();

        Ok([
            format!(
                "build {}: compile_main {} {} | {} || {}",
                bit_code_file.display(),
                source_file.display(),
                dependency_file.display(),
                context_interface_files
                    .values()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
                ninja_dependency_file.display(),
            ),
            format!(
                "  context_options = {}",
                context_interface_files
                    .iter()
                    .map(|(key, path)| format!("-c {} {}", key, path.display()))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            format!("  dyndep = {}", ninja_dependency_file.display()),
            format!("  module_name = {}", target.source().module_name()),
            format!("  srcdep = {}", target.source_file()),
        ]
        .into_iter()
        .chain(self.compile_object_file(&bit_code_file, &object_file))
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
                target.source(),
            ),
        )
        .collect())
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_dependency(
        &self,
        source_file: &std::path::Path,
        bit_code_file: &std::path::Path,
        dependency_file: &std::path::Path,
        ninja_dependency_file: &std::path::Path,
        package_directory: &std::path::Path,
        original_source_file: &FilePath,
        target_source: &app::infra::ModuleTargetSource,
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
            format!("  module_name = {}", target_source.module_name()),
            self.format_in_package_name_variable(target_source.package_name()),
            format!("  srcdep = {original_source_file}"),
        ]
    }

    fn compile_archive(
        &self,
        object_files: &[&FilePath],
        archive_file: &FilePath,
        package_directory: &FilePath,
        package_name: Option<&str>,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let os_path = self
            .file_path_converter
            .convert_to_os_path(package_directory);

        Ok(
            if let Some(crate_info) = ffi_crate_finder::find(&os_path)? {
                let ffi_archive_file = archive_file
                    .parent()
                    .join(&FilePath::new([FFI_ARCHIVE_DIRECTORY]))
                    .join(&archive_file.file_name());

                [
                    format!(
                        "build {}: copy_ffi {}",
                        self.file_path_converter
                            .convert_to_os_path(&ffi_archive_file)
                            .display(),
                        FFI_PHONY_TARGET,
                    ),
                    format!("  source = $ffi_target_dir/{}", crate_info.library_name()),
                    self.format_in_package_name_variable(package_name),
                ]
                .into_iter()
                .chain(self.compile_archive_with_ffi(
                    object_files,
                    archive_file,
                    &ffi_archive_file,
                    package_name,
                )?)
                .collect()
            } else {
                self.compile_archive_without_ffi(object_files, archive_file, package_name)?
            },
        )
    }

    fn compile_archive_without_ffi(
        &self,
        object_files: &[&FilePath],
        archive_file: &FilePath,
        package_name: Option<&str>,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        Ok(vec![
            format!(
                "build {}: ar {}",
                self.file_path_converter
                    .convert_to_os_path(archive_file)
                    .display(),
                self.join_paths(object_files)
            ),
            format!("  package_name = {}", package_name.unwrap_or_default()),
        ])
    }

    fn compile_archive_with_ffi(
        &self,
        object_files: &[&FilePath],
        archive_file: &FilePath,
        ffi_archive_file: &FilePath,
        package_name: Option<&str>,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let ffi_archive_file = self
            .file_path_converter
            .convert_to_os_path(ffi_archive_file);
        let object_files = self.join_paths(object_files);

        Ok(vec![
            format!(
                "build {}: ar_ffi {} {}",
                self.file_path_converter
                    .convert_to_os_path(archive_file)
                    .display(),
                ffi_archive_file.display(),
                &object_files,
            ),
            format!("  ffi_archive_file = {}", ffi_archive_file.display()),
            format!("  object_files = {}", &object_files),
            format!("  package_name = {}", package_name.unwrap_or_default()),
        ])
    }

    fn compile_object_file(&self, bit_code_file: &Path, object_file: &Path) -> Vec<String> {
        let optimized_bit_code_file = bit_code_file
            .with_file_name(format!(
                "{}_opt",
                bit_code_file.file_stem().unwrap().to_string_lossy()
            ))
            .with_extension(self.bit_code_file_extension);

        vec![
            format!(
                "build {}: opt {}",
                optimized_bit_code_file.display(),
                bit_code_file.display(),
            ),
            format!(
                "build {}: llc {}",
                object_file.display(),
                optimized_bit_code_file.display(),
            ),
        ]
    }

    fn join_paths(&self, paths: &[&FilePath]) -> String {
        paths
            .iter()
            .map(|path| {
                format!(
                    "{}",
                    self.file_path_converter.convert_to_os_path(path).display()
                )
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn find_link_script(
        &self,
        system_package_directories: &[PathBuf],
    ) -> Result<PathBuf, Box<dyn Error>> {
        let scripts = system_package_directories
            .iter()
            .flat_map(|directory| {
                package_script_finder::find(directory, self.link_script_basename).transpose()
            })
            .collect::<Result<Vec<_>, _>>()?;

        match scripts.as_slice() {
            [] => Err(InfrastructureError::LinkScriptNotFound.into()),
            [script] => Ok(script.into()),
            _ => Err(InfrastructureError::MultipleLinkScripts(scripts).into()),
        }
    }

    fn format_in_package_name_variable(&self, package_name: Option<&str>) -> String {
        format!(
            "  in_package_name = {}",
            package_name
                .map(|name| "in ".to_owned() + name)
                .unwrap_or_default()
        )
    }
}

fn relative_path(from: &Path, to: &Path) -> PathBuf {
    let from = from.components().collect::<Vec<_>>();
    let to = to.components().collect::<Vec<_>>();

    let common = from
        .iter()
        .zip(to.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let mut result = PathBuf::new();

    for _ in &from[common..] {
        result.push(Component::ParentDir);
    }

    for component in &to[common..] {
        result.push(component);
    }

    result
}

impl app::infra::BuildScriptCompiler for NinjaBuildScriptCompiler {
    fn compile_main(
        &self,
        prelude_interface_files: &[FilePath],
        output_directory: &FilePath,
        target_triple: Option<&str>,
        child_build_script_files: &[FilePath],
        ffi_package_directories: &[FilePath],
    ) -> Result<String, Box<dyn Error>> {
        let output_os_path = self
            .file_path_converter
            .convert_to_os_path(output_directory);

        let ffi_crates = ffi_package_directories
            .iter()
            .filter_map(|directory| {
                let os_path = self.file_path_converter.convert_to_os_path(directory);
                ffi_crate_finder::find(&os_path)
                    .transpose()
                    .map(|result| result.map(|info| (os_path, info)))
            })
            .collect::<Result<Vec<_>, _>>()?;

        if !ffi_crates.is_empty() {
            let members = ffi_crates
                .iter()
                .map(|(_, info)| {
                    format!(
                        "  \"{}\"",
                        relative_path(&output_os_path, info.directory()).display()
                    )
                })
                .collect::<Vec<_>>();

            fs::write(
                output_os_path.join("Cargo.toml"),
                format!(
                    "[workspace]\nresolver = \"2\"\nmembers = [\n{},\n]\n",
                    members.join(",\n")
                ),
            )?;
        }

        Ok([
            "ninja_required_version = 1.10".into(),
            format!("builddir = {}", output_os_path.display()),
        ]
        .into_iter()
        .chain(self.compile_rules(prelude_interface_files, target_triple)?)
        .chain(if ffi_crates.is_empty() {
            vec![format!("build {FFI_PHONY_TARGET}: phony")]
        } else {
            {
                let stamp = output_os_path.join(FFI_BUILD_STAMP);
                vec![
                    format!(
                        "ffi_target_dir = {}/target/$target/release",
                        output_os_path.display()
                    ),
                    format!("build {}: build_ffi", stamp.display()),
                    format!(
                        "  manifest_path = {}",
                        output_os_path.join("Cargo.toml").display()
                    ),
                    format!("build {FFI_PHONY_TARGET}: phony {}", stamp.display()),
                ]
            }
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

    fn compile_modules(
        &self,
        module_targets: &[app::infra::ModuleTarget],
        main_module_target: Option<&app::infra::MainModuleTarget>,
        archive_file: &FilePath,
        package_directory: &FilePath,
    ) -> Result<String, Box<dyn Error>> {
        Ok(self
            .compile_module_targets(module_targets)?
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
                    None,
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
    ) -> Result<String, Box<dyn Error>> {
        Ok(self
            .compile_test_module_targets(module_targets)?
            .into_iter()
            .chain(
                self.compile_archive_without_ffi(
                    &module_targets
                        .iter()
                        .map(|target| target.object_file())
                        .collect::<Vec<_>>(),
                    archive_file,
                    None,
                )?,
            )
            .chain([format!(
                "build {}: compile_package_test_information {}",
                self.file_path_converter
                    .convert_to_os_path(package_test_information_file)
                    .display(),
                self.join_paths(
                    &module_targets
                        .iter()
                        .map(|target| target.test_information_file())
                        .collect::<Vec<_>>()
                )
            )])
            .collect::<Vec<_>>()
            .join("\n")
            + "\n")
    }

    fn compile_external_package(
        &self,
        module_targets: &[app::infra::ModuleTarget],
        archive_file: &FilePath,
        package_directory: &FilePath,
        package_name: &str,
    ) -> Result<String, Box<dyn Error>> {
        Ok(self
            .compile_module_targets(module_targets)?
            .into_iter()
            .chain(
                self.compile_archive(
                    &module_targets
                        .iter()
                        .map(|target| target.object_file())
                        .collect::<Vec<_>>(),
                    archive_file,
                    package_directory,
                    Some(package_name),
                )?,
            )
            .collect::<Vec<_>>()
            .join("\n")
            + "\n")
    }

    fn compile_application(
        &self,
        system_package_directories: &[FilePath],
        archive_files: &[FilePath],
        application_file: &FilePath,
    ) -> Result<String, Box<dyn Error>> {
        let application_file = self
            .file_path_converter
            .convert_to_os_path(application_file);

        Ok([
            "rule link".into(),
            format!(
                "  command = {} -t $target -o $out $in",
                self.find_link_script(
                    &system_package_directories
                        .iter()
                        .map(|directory| self.file_path_converter.convert_to_os_path(directory))
                        .collect::<Vec<_>>()
                )?
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

        Ok([
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

    fn compile_prelude_package(
        &self,
        module_targets: &[app::infra::ModuleTarget],
        archive_file: &FilePath,
        package_directory: &FilePath,
        package_name: &str,
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

                [format!(
                    "build {} {}: compile_prelude {}",
                    bit_code_file.display(),
                    interface_file.display(),
                    source_file.display(),
                )]
                .into_iter()
                .chain(self.compile_object_file(&bit_code_file, &object_file))
            })
            .chain(
                self.compile_archive(
                    &module_targets
                        .iter()
                        .map(|target| target.object_file())
                        .collect::<Vec<_>>(),
                    archive_file,
                    package_directory,
                    Some(package_name),
                )?,
            )
            .collect::<Vec<_>>()
            .join("\n")
            + "\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_relative_path_to_sibling() {
        assert_eq!(
            relative_path(Path::new("/a/b/ffi"), Path::new("/a/b/packages/os/ffi")),
            PathBuf::from("../packages/os/ffi")
        );
    }

    #[test]
    fn compute_relative_path_to_child() {
        assert_eq!(
            relative_path(Path::new("/a/b"), Path::new("/a/b/c/d")),
            PathBuf::from("c/d")
        );
    }

    #[test]
    fn compute_relative_path_to_same() {
        assert_eq!(
            relative_path(Path::new("/a/b"), Path::new("/a/b")),
            PathBuf::from("")
        );
    }
}
