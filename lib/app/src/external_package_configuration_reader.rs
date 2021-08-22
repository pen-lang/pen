use crate::{
    common::file_path_resolver,
    infra::{FilePath, Infrastructure},
};
use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
};

pub fn read_recursively(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
) -> Result<BTreeMap<url::Url, HashMap<String, url::Url>>, Box<dyn Error>> {
    read_dependencies(
        infrastructure,
        &infrastructure
            .package_configuration_reader
            .get_dependencies(package_directory)?,
        output_directory,
    )
}

fn read_dependencies(
    infrastructure: &Infrastructure,
    dependencies: &HashMap<String, url::Url>,
    output_directory: &FilePath,
) -> Result<BTreeMap<url::Url, HashMap<String, url::Url>>, Box<dyn Error>> {
    Ok(dependencies
        .values()
        .map(|url| -> Result<_, Box<dyn Error>> {
            let configuration = infrastructure
                .package_configuration_reader
                .get_dependencies(&file_path_resolver::resolve_package_directory(
                    output_directory,
                    url,
                ))?;

            Ok(vec![(url.clone(), configuration.clone())]
                .into_iter()
                .chain(read_dependencies(
                    infrastructure,
                    &configuration,
                    output_directory,
                )?)
                .collect::<Vec<_>>())
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect())
}
