use crate::{
    external_package_configuration_reader,
    infra::{FilePath, Infrastructure, PackageConfiguration},
};
use petgraph::{algo::toposort, Graph};
use std::{collections::HashMap, error::Error};

pub fn sort(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
) -> Result<Vec<url::Url>, Box<dyn Error>> {
    sort_external_packages(&external_package_configuration_reader::read_recursively(
        infrastructure,
        package_directory,
        output_directory,
    )?)
}

fn sort_external_packages(
    package_configurations: &HashMap<url::Url, PackageConfiguration>,
) -> Result<Vec<url::Url>, Box<dyn std::error::Error>> {
    let mut graph = Graph::<url::Url, ()>::new();
    let mut indices = HashMap::<url::Url, _>::new();

    for external_package in package_configurations.keys() {
        indices.insert(
            external_package.clone(),
            graph.add_node(external_package.clone()),
        );
    }

    for (url, package_configuration) in package_configurations {
        for dependency_url in package_configuration.dependencies.values() {
            graph.add_edge(indices[dependency_url], indices[url], ());
        }
    }

    Ok(toposort(&graph, None)
        .unwrap()
        .into_iter()
        .map(|index| graph[index].clone())
        .collect())
}
