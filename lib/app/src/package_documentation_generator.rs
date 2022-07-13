use crate::{
    common::file_path_resolver,
    infra::{FilePath, Infrastructure},
    module_finder,
};
use parse::{parse, parse_comments};
use std::error::Error;

pub struct DocumentationPackage {
    pub name: String,
    pub url: String,
    pub description: String,
}

pub type DocumentationConfiguration = doc::Configuration;

pub fn generate(
    infrastructure: &Infrastructure,
    package: &DocumentationPackage,
    package_directory: &FilePath,
    configuration: &DocumentationConfiguration,
) -> Result<String, Box<dyn Error>> {
    Ok(doc::generate(
        &doc::Package {
            name: package.name.clone(),
            url: package.name.clone(),
            description: package.name.clone(),
            type_: infrastructure
                .package_configuration_reader
                .read(package_directory)?
                .type_()
                .to_string(),
        },
        &module_finder::find(infrastructure, package_directory)?
            .iter()
            .map(|path| -> Result<_, Box<dyn Error>> {
                Ok((
                    ast::ExternalModulePath::new(
                        &package.name,
                        file_path_resolver::resolve_module_path_components(package_directory, path),
                    )
                    .into(),
                    {
                        let source = infrastructure.file_system.read_to_string(path)?;
                        let path = infrastructure.file_path_displayer.display(path);

                        (parse(&source, &path)?, parse_comments(&source, &path)?)
                    },
                ))
            })
            .collect::<Result<_, _>>()?,
        configuration,
    ))
}
