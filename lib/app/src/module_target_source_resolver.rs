use crate::{
    common::file_path_resolver,
    infra::{FilePath, ModuleTargetSource},
    package_name_formatter,
};

pub fn resolve(
    package_url: Option<&url::Url>,
    package_directory: &FilePath,
    module_file_path: &FilePath,
) -> ModuleTargetSource {
    ModuleTargetSource::new(
        package_url.map(package_name_formatter::format),
        file_path_resolver::resolve_module_path_components(package_directory, module_file_path)
            .join(ast::IDENTIFIER_SEPARATOR),
    )
}
