use super::build_infrastructure::BuildInfrastructure;
use super::module_build_target::ModuleBuildTarget;
use super::module_finder;
use crate::infra::FilePath;
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};

const MODULE_PREFIX_COMPONENT_SEPARATOR: &str = ".";

pub fn build_package(
    infrastructure: &BuildInfrastructure,
    package_prefix: &str,
    package_directory_path: &FilePath,
    output_directory_path: &FilePath,
) -> Result<(), Box<dyn Error>> {
    let object_directory_path = output_directory_path.join(&FilePath::new(vec![
        infrastructure.file_path_configuration.object_directory,
    ]));

    infrastructure.builder.build(
        package_prefix,
        &module_finder::find_modules(infrastructure, package_directory_path)?
            .iter()
            .map(|source_file_path| {
                let module_prefix =
                    calculate_module_prefix(package_directory_path, source_file_path);

                ModuleBuildTarget::new(
                    &module_prefix,
                    source_file_path.clone(),
                    object_directory_path.join(
                        &FilePath::new(vec![calculate_module_id(package_prefix, &module_prefix)])
                            .with_extension(
                                infrastructure.file_path_configuration.object_file_extension,
                            ),
                    ),
                )
            })
            .collect::<Vec<_>>(),
        output_directory_path,
    )?;

    Ok(())
}

fn calculate_module_prefix(
    package_directory_path: &FilePath,
    source_file_path: &FilePath,
) -> String {
    source_file_path
        .relative_to(package_directory_path)
        .with_extension("")
        .components()
        .collect::<Vec<_>>()
        .join(MODULE_PREFIX_COMPONENT_SEPARATOR)
        + MODULE_PREFIX_COMPONENT_SEPARATOR
}

fn calculate_module_id(package_prefix: &str, module_prefix: &str) -> String {
    let mut hasher = DefaultHasher::new();

    package_prefix.hash(&mut hasher);
    module_prefix.hash(&mut hasher);

    format!("{:x}", hasher.finish())
}
