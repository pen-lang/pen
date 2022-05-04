use crate::{
    common::file_path_resolver,
    infra::{FilePath, ModuleTargetSource},
};

pub fn resolve(
    package_url: Option<&url::Url>,
    package_directory: &FilePath,
    module_file_path: &FilePath,
) -> ModuleTargetSource {
    ModuleTargetSource::new(
        package_url.map(|url| url.to_string()),
        file_path_resolver::resolve_module_path_components(package_directory, module_file_path)
            .join(ast::IDENTIFIER_SEPARATOR),
    )
}
