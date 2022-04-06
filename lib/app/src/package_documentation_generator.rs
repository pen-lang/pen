use crate::{
    common::file_path_resolver,
    infra::{FilePath, Infrastructure},
    module_finder,
};
use parse::{parse, parse_comments};
use std::error::Error;

pub type PackageDocumentation = doc::Package;

pub type DocumentationConfiguration = doc::Configuration;

pub fn generate(
    infrastructure: &Infrastructure,
    package: &PackageDocumentation,
    package_directory: &FilePath,
    configuration: &DocumentationConfiguration,
) -> Result<String, Box<dyn Error>> {
    Ok(doc::generate(
        package,
        &module_finder::find(infrastructure, package_directory)?
            .iter()
            .map(|path| -> Result<_, Box<dyn Error>> {
                let source = infrastructure.file_system.read_to_string(path)?;

                Ok((
                    ast::ExternalModulePath::new(
                        &package.name,
                        file_path_resolver::resolve_module_path_components(package_directory, path),
                    )
                    .into(),
                    {
                        let path = infrastructure.file_path_displayer.display(path);

                        (parse(&source, &path)?, parse_comments(&source, &path)?)
                    },
                ))
            })
            .collect::<Result<_, _>>()?,
        configuration,
    ))
}
