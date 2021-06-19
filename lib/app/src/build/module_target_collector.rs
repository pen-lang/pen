use super::build_infrastructure::BuildInfrastructure;
use super::module_finder;
use super::module_target::ModuleTarget;
use crate::infra::FilePath;
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};

const OBJECT_DIRECTORY: &str = "objects";
const MODULE_PREFIX_COMPONENT_SEPARATOR: &str = ".";

pub fn collect_module_targets(
    infrastructure: &BuildInfrastructure,
    package_prefix: &str,
    package_directory: &FilePath,
    output_directory: &FilePath,
) -> Result<Vec<ModuleTarget>, Box<dyn Error>> {
    let object_directory = output_directory.join(&FilePath::new(vec![OBJECT_DIRECTORY]));

    Ok(
        module_finder::find_modules(infrastructure, package_directory)?
            .iter()
            .map(|source_file| {
                let module_prefix = calculate_module_prefix(package_directory, source_file);
                let target_file = object_directory.join(&FilePath::new(vec![calculate_module_id(
                    package_prefix,
                    &module_prefix,
                )]));

                ModuleTarget::new(
                    package_prefix,
                    &module_prefix,
                    source_file.clone(),
                    target_file.with_extension(
                        infrastructure.file_path_configuration.object_file_extension,
                    ),
                    target_file.with_extension(
                        infrastructure
                            .file_path_configuration
                            .interface_file_extension,
                    ),
                )
            })
            .collect::<Vec<_>>(),
    )
}

fn calculate_module_prefix(package_directory: &FilePath, source_file: &FilePath) -> String {
    source_file
        .relative_to(package_directory)
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
