use crate::{
    common::file_path_resolver,
    infra::{FilePath, Infrastructure},
    module_finder,
};
use parse::{parse, parse_comments};
use std::error::Error;

pub fn generate(
    infrastructure: &Infrastructure,
    package_name: &str,
    package_directory: &FilePath,
    language: &str,
) -> Result<String, Box<dyn Error>> {
    Ok(doc::generate(
        package_name,
        &module_finder::find(infrastructure, package_directory)?
            .iter()
            .map(|path| -> Result<_, Box<dyn Error>> {
                let source = infrastructure.file_system.read_to_string(&path)?;

                Ok((
                    ast::ExternalModulePath::new(
                        package_name,
                        file_path_resolver::resolve_module_path_components(package_directory, path),
                    )
                    .into(),
                    {
                        let path = infrastructure.file_path_displayer.display(&path);

                        (parse(&source, &path)?, parse_comments(&source, &path)?)
                    },
                ))
            })
            .collect::<Result<_, _>>()?,
        language,
    ))
}
