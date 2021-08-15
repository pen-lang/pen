use crate::{
    common::file_path_resolver,
    infra::{FilePath, Infrastructure, PackageConfiguration},
};
use std::{collections::HashMap, error::Error};

pub fn read_recursively(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
) -> Result<HashMap<url::Url, PackageConfiguration>, Box<dyn Error>> {
    Ok(read_dependencies(
        infrastructure,
        &infrastructure
            .package_configuration_reader
            .read(package_directory)?,
        output_directory,
    )?)
}

pub fn read_dependencies(
    infrastructure: &Infrastructure,
    configuration: &PackageConfiguration,
    output_directory: &FilePath,
) -> Result<HashMap<url::Url, PackageConfiguration>, Box<dyn Error>> {
    Ok(configuration
        .dependencies
        .values()
        .map(|url| -> Result<_, Box<dyn Error>> {
            let configuration = infrastructure.package_configuration_reader.read(
                &file_path_resolver::resolve_package_directory(output_directory, url),
            )?;

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
