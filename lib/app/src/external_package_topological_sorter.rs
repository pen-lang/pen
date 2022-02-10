use crate::PackageConfiguration;
use petgraph::{algo::toposort, Graph};
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

    // TODO Return an error on cycle.
    Ok(toposort(&graph, None)
        .unwrap()
        .into_iter()
        .map(|index| graph[index].clone())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_packages() {
        assert_eq!(
            sort(
                &[
                    (
                        url::Url::parse("file:///foo").unwrap(),
                        PackageConfiguration::new(Default::default(), false)
                    ),
                    (
                        url::Url::parse("file:///bar").unwrap(),
                        PackageConfiguration::new(
                            [
                                ("Foo".into(), url::Url::parse("file:///foo").unwrap()),
                                ("Baz".into(), url::Url::parse("file:///baz").unwrap())
                            ]
                            .into_iter()
                            .collect(),
                            false
                        )
                    ),
                    (
                        url::Url::parse("file:///baz").unwrap(),
                        PackageConfiguration::new(
                            [("Foo".into(), url::Url::parse("file:///foo").unwrap()),]
                                .into_iter()
                                .collect(),
                            false
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
