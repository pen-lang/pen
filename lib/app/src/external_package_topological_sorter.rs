use crate::{
    external_package_configuration_reader,
    infra::{FilePath, Infrastructure},
};
use petgraph::{algo::toposort, Graph};
use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
};

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
    dependencies: &BTreeMap<url::Url, HashMap<String, url::Url>>,
) -> Result<Vec<url::Url>, Box<dyn std::error::Error>> {
    let mut graph = Graph::<url::Url, ()>::new();
    let mut indices = HashMap::<url::Url, _>::new();

    for external_package in dependencies.keys() {
        indices.insert(
            external_package.clone(),
            graph.add_node(external_package.clone()),
        );
    }

    for (url, dependencies) in dependencies {
        for dependency_url in dependencies.values() {
            graph.add_edge(indices[dependency_url], indices[url], ());
        }
    }

    Ok(toposort(&graph, None)
        .unwrap()
        .into_iter()
        .map(|index| graph[index].clone())
        .collect())
}
