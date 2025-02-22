use crate::{PackageConfiguration, error::ApplicationError};
use petgraph::{Graph, algo::toposort};
use std::{collections::BTreeMap, error::Error};

pub fn sort(
    external_package_configurations: &BTreeMap<url::Url, PackageConfiguration>,
) -> Result<Vec<url::Url>, Box<dyn Error>> {
    let mut graph = Graph::<url::Url, ()>::new();
    let mut indices = BTreeMap::<url::Url, _>::new();

    for external_package in external_package_configurations.keys() {
        indices.insert(
            external_package.clone(),
            graph.add_node(external_package.clone()),
        );
    }

    for (url, configuration) in external_package_configurations {
        for dependency_url in configuration.dependencies().values() {
            graph.add_edge(indices[url], indices[dependency_url], ());
        }
    }

    Ok(toposort(&graph, None)
        .map_err(|_| ApplicationError::PackageDependencyCycle)?
        .into_iter()
        .map(|index| graph[index].clone())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package_configuration::PackageType;

    #[test]
    fn sort_packages() {
        assert_eq!(
            sort(
                &[
                    (
                        url::Url::parse("file:///foo").unwrap(),
                        PackageConfiguration::new(PackageType::Application, Default::default())
                    ),
                    (
                        url::Url::parse("file:///bar").unwrap(),
                        PackageConfiguration::new(
                            PackageType::Application,
                            [
                                ("Foo".into(), url::Url::parse("file:///foo").unwrap()),
                                ("Baz".into(), url::Url::parse("file:///baz").unwrap())
                            ]
                            .into_iter()
                            .collect(),
                        )
                    ),
                    (
                        url::Url::parse("file:///baz").unwrap(),
                        PackageConfiguration::new(
                            PackageType::Application,
                            [("Foo".into(), url::Url::parse("file:///foo").unwrap()),]
                                .into_iter()
                                .collect(),
                        )
                    )
                ]
                .into_iter()
                .collect()
            )
            .unwrap(),
            vec![
                url::Url::parse("file:///bar").unwrap(),
                url::Url::parse("file:///baz").unwrap(),
                url::Url::parse("file:///foo").unwrap(),
            ]
        );
    }
}
