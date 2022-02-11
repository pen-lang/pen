use crate::{
    common::file_path_resolver,
    infra::{FilePath, Infrastructure},
    package_configuration::PackageConfiguration,
};
use std::{collections::BTreeMap, error::Error};

pub fn read_main(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
) -> Result<BTreeMap<String, (url::Url, PackageConfiguration)>, Box<dyn Error>> {
    infrastructure
        .package_configuration_reader
        .read(package_directory)?
        .dependencies()
        .into_iter()
        .map(|(key, url)| -> Result<_, Box<dyn Error>> {
            Ok((
                key.clone(),
                (
                    url.clone(),
                    infrastructure.package_configuration_reader.read(
                        &file_path_resolver::resolve_package_directory(output_directory, url),
                    )?,
                ),
            ))
        })
        .collect::<Result<_, _>>()
}

pub fn read_all(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
) -> Result<BTreeMap<url::Url, PackageConfiguration>, Box<dyn Error>> {
    read_dependencies(
        infrastructure,
        &infrastructure
            .package_configuration_reader
            .read(package_directory)?,
        output_directory,
    )
}

fn read_dependencies(
    infrastructure: &Infrastructure,
    configuration: &PackageConfiguration,
    output_directory: &FilePath,
) -> Result<BTreeMap<url::Url, PackageConfiguration>, Box<dyn Error>> {
    Ok(configuration
        .dependencies()
        .values()
        .map(|url| -> Result<_, Box<dyn Error>> {
            let configuration = infrastructure.package_configuration_reader.read(
                &file_path_resolver::resolve_package_directory(output_directory, url),
            )?;
            let dependencies = read_dependencies(infrastructure, &configuration, output_directory)?;

            Ok([(url.clone(), configuration)]
                .into_iter()
                .chain(dependencies)
                .collect::<Vec<_>>())
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect())
}
