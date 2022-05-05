use crate::{
    common::file_path_resolver,
    infra::{FilePath, Infrastructure, ModuleTarget},
    module_finder, module_target_source_resolver,
};
use std::error::Error;

pub fn collect_module_targets(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
    package_url: Option<&url::Url>,
    output_directory: &FilePath,
) -> Result<Vec<ModuleTarget>, Box<dyn Error>> {
    Ok(module_finder::find(infrastructure, package_directory)?
        .iter()
        .map(|source_file| {
            ModuleTarget::new(
                package_directory.clone(),
                source_file.clone(),
                file_path_resolver::resolve_object_file(
                    output_directory,
                    source_file,
                    &infrastructure.file_path_configuration,
                ),
                file_path_resolver::resolve_interface_file(
                    output_directory,
                    source_file,
                    &infrastructure.file_path_configuration,
                ),
                module_target_source_resolver::resolve(package_url, package_directory, source_file),
            )
        })
        .collect::<Vec<_>>())
}
